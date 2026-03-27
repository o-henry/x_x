//! encode and decode the frames for the mux protocol.
//! The frames include the length of a PDU as well as an identifier
//! that informs us how to decode it.  The length, ident and serial
//! number are encoded using a variable length integer encoding.
//! Rather than rely solely on serde to serialize and deserialize an
//! enum, we encode the enum variants with a version/identifier tag
//! for ourselves.  This will make it a little easier to manage
//! client and server instances that are built from different versions
//! of this code; in this way the client and server can more gracefully
//! manage unknown enum variants.
#![allow(dead_code)]
#![allow(clippy::drop_non_drop)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::range_plus_one)]

use anyhow::{bail, Context as _, Error};
use config::keyassignment::{PaneDirection, ScrollbackEraseMode};
use mux::client::{ClientId, ClientInfo};
use mux::pane::PaneId;
use mux::renderable::{RenderableDimensions, StableCursorPosition};
use mux::tab::{PaneNode, SerdeUrl, SplitRequest, TabId};
use mux::window::WindowId;
use portable_pty::CommandBuilder;
use rangeset::*;
use serde::{Deserialize, Serialize};
use smol::io::AsyncWriteExt;
use smol::prelude::*;
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Cursor;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;
use termwiz::hyperlink::Hyperlink;
use termwiz::image::{ImageData, TextureCoordinate};
use termwiz::surface::{Line, SequenceNo};
use thiserror::Error;
use wezterm_term::color::ColorPalette;
use wezterm_term::{Alert, ClipboardSelection, StableRowIndex, TerminalSize};

#[derive(Error, Debug)]
#[error("Corrupt Response: {0}")]
pub struct CorruptResponse(String);

/// Returns the encoded length of the leb128 representation of value
fn encoded_length(value: u64) -> usize {
    struct NullWrite {}
    impl std::io::Write for NullWrite {
        fn write(&mut self, buf: &[u8]) -> std::result::Result<usize, std::io::Error> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
            Ok(())
        }
    }

    leb128::write::unsigned(&mut NullWrite {}, value).unwrap()
}

const COMPRESSED_MASK: u64 = 1 << 63;

fn encode_raw_as_vec(
    ident: u64,
    serial: u64,
    data: &[u8],
    is_compressed: bool,
) -> anyhow::Result<Vec<u8>> {
    let len = data.len() + encoded_length(ident) + encoded_length(serial);
    let masked_len = if is_compressed {
        (len as u64) | COMPRESSED_MASK
    } else {
        len as u64
    };

    // Double-buffer the data; since we run with nodelay enabled, it is
    // desirable for the write to be a single packet (or at least, for
    // the header portion to go out in a single packet)
    let mut buffer = Vec::with_capacity(len + encoded_length(masked_len));

    leb128::write::unsigned(&mut buffer, masked_len).context("writing pdu len")?;
    leb128::write::unsigned(&mut buffer, serial).context("writing pdu serial")?;
    leb128::write::unsigned(&mut buffer, ident).context("writing pdu ident")?;
    buffer.extend_from_slice(data);

    if is_compressed {
        metrics::histogram!("pdu.encode.compressed.size").record(buffer.len() as f64);
    } else {
        metrics::histogram!("pdu.encode.size").record(buffer.len() as f64);
    }

    Ok(buffer)
}

/// Encode a frame.  If the data is compressed, the high bit of the length
/// is set to indicate that.  The data written out has the format:
/// tagged_len: leb128  (u64 msb is set if data is compressed)
/// serial: leb128
/// ident: leb128
/// data bytes
fn encode_raw<W: std::io::Write>(
    ident: u64,
    serial: u64,
    data: &[u8],
    is_compressed: bool,
    mut w: W,
) -> anyhow::Result<usize> {
    let buffer = encode_raw_as_vec(ident, serial, data, is_compressed)?;
    w.write_all(&buffer).context("writing pdu data buffer")?;
    Ok(buffer.len())
}

async fn encode_raw_async<W: Unpin + AsyncWriteExt>(
    ident: u64,
    serial: u64,
    data: &[u8],
    is_compressed: bool,
    w: &mut W,
) -> anyhow::Result<usize> {
    let buffer = encode_raw_as_vec(ident, serial, data, is_compressed)?;
    w.write_all(&buffer)
        .await
        .context("writing pdu data buffer")?;
    Ok(buffer.len())
}

/// Read a single leb128 encoded value from the stream
async fn read_u64_async<R>(r: &mut R) -> anyhow::Result<u64>
where
    R: Unpin + AsyncRead + std::fmt::Debug,
{
    let mut buf = vec![];
    loop {
        let mut byte = [0u8];
        let nread = r.read(&mut byte).await?;
        if nread == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "EOF while reading leb128 encoded value",
            )
            .into());
        }
        buf.push(byte[0]);

        match leb128::read::unsigned(&mut buf.as_slice()) {
            Ok(n) => {
                return Ok(n);
            }
            Err(leb128::read::Error::IoError(_)) => continue,
            Err(leb128::read::Error::Overflow) => anyhow::bail!("leb128 is too large"),
        }
    }
}

/// Read a single leb128 encoded value from the stream
fn read_u64<R: std::io::Read>(mut r: R) -> anyhow::Result<u64> {
    leb128::read::unsigned(&mut r)
        .map_err(|err| match err {
            leb128::read::Error::IoError(ioerr) => anyhow::Error::new(ioerr),
            err => anyhow::Error::new(err),
        })
        .context("reading leb128")
}

#[derive(Debug)]
struct Decoded {
    ident: u64,
    serial: u64,
    data: Vec<u8>,
    is_compressed: bool,
}

/// Decode a frame.
/// See encode_raw() for the frame format.
async fn decode_raw_async<R: Unpin + AsyncRead + std::fmt::Debug>(
    r: &mut R,
    max_serial: Option<u64>,
) -> anyhow::Result<Decoded> {
    let len = read_u64_async(r)
        .await
        .context("decode_raw_async failed to read PDU length")?;
    let (len, is_compressed) = if (len & COMPRESSED_MASK) != 0 {
        (len & !COMPRESSED_MASK, true)
    } else {
        (len, false)
    };
    let serial = read_u64_async(r)
        .await
        .context("decode_raw_async failed to read PDU serial")?;
    if let Some(max_serial) = max_serial {
        if serial > max_serial && max_serial > 0 {
            return Err(CorruptResponse(format!(
                "decode_raw_async: serial {serial} is implausibly large \
                (bigger than {max_serial})"
            ))
            .into());
        }
    }
    let ident = read_u64_async(r)
        .await
        .context("decode_raw_async failed to read PDU ident")?;
    let data_len =
        match (len as usize).overflowing_sub(encoded_length(ident) + encoded_length(serial)) {
            (_, true) => {
                return Err(CorruptResponse(format!(
                    "decode_raw_async: sizes don't make sense: \
                    len:{len} serial:{serial} (enc={}) ident:{ident} (enc={})",
                    encoded_length(serial),
                    encoded_length(ident)
                ))
                .into());
            }
            (data_len, false) => data_len,
        };

    if is_compressed {
        metrics::histogram!("pdu.decode.compressed.size").record(data_len as f64);
    } else {
        metrics::histogram!("pdu.decode.size").record(data_len as f64);
    }

    let mut data = vec![0u8; data_len];
    r.read_exact(&mut data).await.with_context(|| {
        format!(
            "decode_raw_async failed to read {} bytes of data \
            for PDU of length {} with serial={} ident={}",
            data_len, len, serial, ident
        )
    })?;
    Ok(Decoded {
        ident,
        serial,
        data,
        is_compressed,
    })
}

/// Decode a frame.
/// See encode_raw() for the frame format.
fn decode_raw<R: std::io::Read>(mut r: R) -> anyhow::Result<Decoded> {
    let len = read_u64(r.by_ref()).context("reading PDU length")?;
    let (len, is_compressed) = if (len & COMPRESSED_MASK) != 0 {
        (len & !COMPRESSED_MASK, true)
    } else {
        (len, false)
    };
    let serial = read_u64(r.by_ref()).context("reading PDU serial")?;
    let ident = read_u64(r.by_ref()).context("reading PDU ident")?;
    let data_len =
        match (len as usize).overflowing_sub(encoded_length(ident) + encoded_length(serial)) {
            (_, true) => {
                anyhow::bail!(
                    "sizes don't make sense: len:{} serial:{} (enc={}) ident:{} (enc={})",
                    len,
                    serial,
                    encoded_length(serial),
                    ident,
                    encoded_length(ident)
                );
            }
            (data_len, false) => data_len,
        };

    if is_compressed {
        metrics::histogram!("pdu.decode.compressed.size").record(data_len as f64);
    } else {
        metrics::histogram!("pdu.decode.size").record(data_len as f64);
    }

    let mut data = vec![0u8; data_len];
    r.read_exact(&mut data).with_context(|| {
        format!(
            "reading {} bytes of data for PDU of length {} with serial={} ident={}",
            data_len, len, serial, ident
        )
    })?;
    Ok(Decoded {
        ident,
        serial,
        data,
        is_compressed,
    })
}

#[derive(Debug, PartialEq)]
pub struct DecodedPdu {
    pub serial: u64,
    pub pdu: Pdu,
}

/// If the serialized size is larger than this, then we'll consider compressing it
const COMPRESS_THRESH: usize = 32;

fn serialize<T: serde::Serialize>(t: &T) -> Result<(Vec<u8>, bool), Error> {
    let mut uncompressed = Vec::new();
    let mut encode = varbincode::Serializer::new(&mut uncompressed);
    t.serialize(&mut encode)?;

    if uncompressed.len() <= COMPRESS_THRESH {
        return Ok((uncompressed, false));
    }
    // It's a little heavy; let's try compressing it
    let mut compressed = Vec::new();
    let mut compress = zstd::Encoder::new(&mut compressed, zstd::DEFAULT_COMPRESSION_LEVEL)?;
    let mut encode = varbincode::Serializer::new(&mut compress);
    t.serialize(&mut encode)?;
    drop(encode);
    compress.finish()?;

    log::debug!(
        "serialized+compress len {} vs {}",
        compressed.len(),
        uncompressed.len()
    );

    if compressed.len() < uncompressed.len() {
        Ok((compressed, true))
    } else {
        Ok((uncompressed, false))
    }
}

fn deserialize<T: serde::de::DeserializeOwned, R: std::io::Read>(
    mut r: R,
    is_compressed: bool,
) -> Result<T, Error> {
    if is_compressed {
        let mut decompress = zstd::Decoder::new(r)?;
        let mut decode = varbincode::Deserializer::new(&mut decompress);
        serde::Deserialize::deserialize(&mut decode).map_err(Into::into)
    } else {
        let mut decode = varbincode::Deserializer::new(&mut r);
        serde::Deserialize::deserialize(&mut decode).map_err(Into::into)
    }
}

macro_rules! pdu {
    ($( $name:ident:$vers:expr),* $(,)?) => {
        #[derive(PartialEq, Debug)]
        pub enum Pdu {
            Invalid{ident: u64},
            $(
                $name($name)
            ,)*
        }

        impl Pdu {
            pub fn encode<W: std::io::Write>(&self, w: W, serial: u64) -> Result<(), Error> {
                match self {
                    Pdu::Invalid{..} => bail!("attempted to serialize Pdu::Invalid"),
                    $(
                        Pdu::$name(s) => {
                            let (data, is_compressed) = serialize(s)?;
                            let encoded_size = encode_raw($vers, serial, &data, is_compressed, w)?;
                            log::debug!("encode {} size={encoded_size}", stringify!($name));
                            metrics::histogram!("pdu.size", "pdu" => stringify!($name)).record(encoded_size as f64);
                            metrics::histogram!("pdu.size.rate", "pdu" => stringify!($name)).record(encoded_size as f64);
                            Ok(())
                        }
                    ,)*
                }
            }

            pub async fn encode_async<W: Unpin + AsyncWriteExt>(&self, w: &mut W, serial: u64) -> Result<(), Error> {
                match self {
                    Pdu::Invalid{..} => bail!("attempted to serialize Pdu::Invalid"),
                    $(
                        Pdu::$name(s) => {
                            let (data, is_compressed) = serialize(s)?;
                            let encoded_size = encode_raw_async($vers, serial, &data, is_compressed, w).await?;
                            log::debug!("encode_async {} size={encoded_size}", stringify!($name));
                            metrics::histogram!("pdu.size", "pdu" => stringify!($name)).record(encoded_size as f64);
                            metrics::histogram!("pdu.size.rate", "pdu" => stringify!($name)).record(encoded_size as f64);
                            Ok(())
                        }
                    ,)*
                }
            }

            pub fn pdu_name(&self) -> &'static str {
                match self {
                    Pdu::Invalid{..} => "Invalid",
                    $(
                        Pdu::$name(_) => {
                            stringify!($name)
                        }
                    ,)*
                }
            }

            pub fn decode<R: std::io::Read>(r: R) -> Result<DecodedPdu, Error> {
                let decoded = decode_raw(r).context("decoding a PDU")?;
                match decoded.ident {
                    $(
                        $vers => {
                            metrics::histogram!("pdu.size", "pdu" => stringify!($name)).record(decoded.data.len() as f64);
                            metrics::histogram!("pdu.size.rate", "pdu" => stringify!($name)).record(decoded.data.len() as f64);
                            Ok(DecodedPdu {
                                serial: decoded.serial,
                                pdu: Pdu::$name(deserialize(decoded.data.as_slice(), decoded.is_compressed)?)
                            })
                        }
                    ,)*
                    _ => {
                        metrics::histogram!("pdu.size", "pdu" => "??").record(decoded.data.len() as f64);
                        metrics::histogram!("pdu.size.rate", "pdu" => "??").record(decoded.data.len() as f64);
                        Ok(DecodedPdu {
                            serial: decoded.serial,
                            pdu: Pdu::Invalid{ident:decoded.ident}
                        })
                    }
                }
            }

            pub async fn decode_async<R>(r: &mut R, max_serial: Option<u64>) -> Result<DecodedPdu, Error>
                where R: std::marker::Unpin,
                      R: AsyncRead,
                      R: std::fmt::Debug
            {
                let decoded = decode_raw_async(r, max_serial).await.context("decoding a PDU")?;
                match decoded.ident {
                    $(
                        $vers => {
                            metrics::histogram!("pdu.size", "pdu" => stringify!($name)).record(decoded.data.len() as f64);
                            Ok(DecodedPdu {
                                serial: decoded.serial,
                                pdu: Pdu::$name(deserialize(decoded.data.as_slice(), decoded.is_compressed)?)
                            })
                        }
                    ,)*
                    _ => {
                        metrics::histogram!("pdu.size", "pdu" => "??").record(decoded.data.len() as f64);
                        Ok(DecodedPdu {
                            serial: decoded.serial,
                            pdu: Pdu::Invalid{ident:decoded.ident}
                        })
                    }
                }
            }
        }
    }
}

/// The overall version of the codec.
/// This must be bumped when backwards incompatible changes
/// are made to the types and protocol.
pub const CODEC_VERSION: usize = 46;

// Defines the Pdu enum.
// Each struct has an explicit identifying number.
// This allows removal of obsolete structs,
// and defining newer structs as the protocol evolves.
pdu! {
    ErrorResponse: 0,
    Ping: 1,
    Pong: 2,
    ListPanes: 3,
    ListPanesResponse: 4,
    SpawnResponse: 8,
    WriteToPane: 9,
    UnitResponse: 10,
    SendKeyDown: 11,
    SendMouseEvent: 12,
    SendPaste: 13,
    Resize: 14,
    SetClipboard: 20,
    GetLines: 22,
    GetLinesResponse: 23,
    GetPaneRenderChanges: 24,
    GetPaneRenderChangesResponse: 25,
    GetCodecVersion: 26,
    GetCodecVersionResponse: 27,
    GetTlsCreds: 28,
    GetTlsCredsResponse: 29,
    LivenessResponse: 30,
    SearchScrollbackRequest: 31,
    SearchScrollbackResponse: 32,
    SetPaneZoomed: 33,
    SplitPane: 34,
    KillPane: 35,
    SpawnV2: 36,
    PaneRemoved: 37,
    SetPalette: 38,
    NotifyAlert: 39,
    SetClientId: 40,
    GetClientList: 41,
    GetClientListResponse: 42,
    SetWindowWorkspace: 43,
    WindowWorkspaceChanged: 44,
    SetFocusedPane: 45,
    GetImageCell: 46,
    GetImageCellResponse: 47,
    MovePaneToNewTab: 48,
    MovePaneToNewTabResponse: 49,
    ActivatePaneDirection: 50,
    GetPaneRenderableDimensions: 51,
    GetPaneRenderableDimensionsResponse: 52,
    PaneFocused: 53,
    TabResized: 54,
    TabAddedToWindow: 55,
    TabTitleChanged: 56,
    WindowTitleChanged: 57,
    RenameWorkspace: 58,
    EraseScrollbackRequest: 59,
    GetPaneDirection: 60,
    GetPaneDirectionResponse: 61,
    AdjustPaneSize: 62,
    CreateNotification: 63,
    CreateNotificationResponse: 64,
    ListNotifications: 65,
    ListNotificationsResponse: 66,
    ClearNotifications: 67,
    ClearNotificationsResponse: 68,
    MarkNotificationsRead: 69,
    MarkNotificationsUnread: 70,
    MarkNotificationsResponse: 71,
    JumpUnread: 72,
    JumpUnreadResponse: 73,
    IdentifyNotificationTarget: 74,
    IdentifyNotificationTargetResponse: 75,
    GetNotificationCapabilities: 76,
    GetNotificationCapabilitiesResponse: 77,
    SetWorkspaceStatus: 78,
    SetWorkspaceStatusResponse: 79,
    ClearWorkspaceStatus: 80,
    ClearWorkspaceStatusResponse: 81,
    ListWorkspaceStatus: 82,
    ListWorkspaceStatusResponse: 83,
    SetWorkspaceProgress: 84,
    SetWorkspaceProgressResponse: 85,
    ClearWorkspaceProgress: 86,
    ClearWorkspaceProgressResponse: 87,
    AppendWorkspaceLog: 88,
    AppendWorkspaceLogResponse: 89,
    ClearWorkspaceLog: 90,
    ClearWorkspaceLogResponse: 91,
    ListWorkspaceLog: 92,
    ListWorkspaceLogResponse: 93,
    ListTaskPanes: 94,
    ListTaskPanesResponse: 95,
    SetRemainOnExit: 96,
    SetRemainOnExitResponse: 97,
    RerunPane: 98,
    RerunPaneResponse: 99,
    RespawnPane: 100,
    RespawnPaneResponse: 101,
    SilenceWatchdog: 102,
    SilenceWatchdogResponse: 103,
    PipePane: 104,
    PipePaneResponse: 105,
    TaskPaneChanged: 106,
}

impl Pdu {
    /// Returns true if this type of Pdu represents action taken
    /// directly by a user, rather than background traffic on
    /// a live connection
    pub fn is_user_input(&self) -> bool {
        match self {
            Self::WriteToPane(_)
            | Self::SendKeyDown(_)
            | Self::SendMouseEvent(_)
            | Self::SendPaste(_)
            | Self::Resize(_)
            | Self::SetClipboard(_)
            | Self::SetPaneZoomed(_)
            | Self::SpawnV2(_) => true,
            _ => false,
        }
    }

    pub fn stream_decode(buffer: &mut Vec<u8>) -> anyhow::Result<Option<DecodedPdu>> {
        let mut cursor = Cursor::new(buffer.as_slice());
        match Self::decode(&mut cursor) {
            Ok(decoded) => {
                let consumed = cursor.position() as usize;
                let remain = buffer.len() - consumed;
                // Remove `consumed` bytes from the start of the vec.
                // This is safe because the vec is just bytes and we are
                // constrained the offsets accordingly.
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        buffer.as_ptr().add(consumed),
                        buffer.as_mut_ptr(),
                        remain,
                    );
                }
                buffer.truncate(remain);
                Ok(Some(decoded))
            }
            Err(err) => {
                if let Some(ioerr) = err.root_cause().downcast_ref::<std::io::Error>() {
                    match ioerr.kind() {
                        std::io::ErrorKind::UnexpectedEof | std::io::ErrorKind::WouldBlock => {
                            return Ok(None);
                        }
                        _ => {}
                    }
                } else {
                    log::error!("not an ioerror in stream_decode: {:?}", err);
                }
                Err(err)
            }
        }
    }

    pub fn try_read_and_decode<R: std::io::Read>(
        r: &mut R,
        buffer: &mut Vec<u8>,
    ) -> anyhow::Result<Option<DecodedPdu>> {
        loop {
            if let Some(decoded) =
                Self::stream_decode(buffer).context("stream_decode of buffer for PDU")?
            {
                return Ok(Some(decoded));
            }

            let mut buf = [0u8; 4096];
            let size = match r.read(&mut buf) {
                Ok(size) => size,
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::WouldBlock {
                        return Ok(None);
                    }
                    return Err(err.into());
                }
            };
            if size == 0 {
                return Err(
                    std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "End Of File").into(),
                );
            }

            buffer.extend_from_slice(&buf[0..size]);
        }
    }

    pub fn pane_id(&self) -> Option<PaneId> {
        match self {
            Pdu::GetPaneRenderChangesResponse(GetPaneRenderChangesResponse { pane_id, .. })
            | Pdu::SetPalette(SetPalette { pane_id, .. })
            | Pdu::NotifyAlert(NotifyAlert { pane_id, .. })
            | Pdu::SetClipboard(SetClipboard { pane_id, .. })
            | Pdu::PaneFocused(PaneFocused { pane_id })
            | Pdu::PaneRemoved(PaneRemoved { pane_id }) => Some(*pane_id),
            Pdu::TaskPaneChanged(TaskPaneChanged { pane_id }) => Some(*pane_id),
            _ => None,
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct UnitResponse {}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ErrorResponse {
    pub reason: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct GetCodecVersion {}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct GetCodecVersionResponse {
    pub codec_vers: usize,
    pub version_string: String,
    pub executable_path: PathBuf,
    pub config_file_path: Option<PathBuf>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Ping {}
#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Pong {}

/// Requests a client certificate to authenticate against
/// the TLS based server
#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct GetTlsCreds {}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct GetTlsCredsResponse {
    /// The signing certificate
    pub ca_cert_pem: String,
    /// A client authentication certificate and private
    /// key, PEM encoded
    pub client_cert_pem: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ListPanes {}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ListPanesResponse {
    pub tabs: Vec<PaneNode>,
    pub tab_titles: Vec<String>,
    pub window_titles: HashMap<WindowId, String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SplitPane {
    pub pane_id: PaneId,
    pub split_request: SplitRequest,
    pub command: Option<CommandBuilder>,
    pub command_dir: Option<String>,
    pub domain: config::keyassignment::SpawnTabDomain,
    /// Instead of spawning a command, move the specified
    /// pane into the new split target
    pub move_pane_id: Option<PaneId>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct MovePaneToNewTab {
    pub pane_id: PaneId,
    pub window_id: Option<WindowId>,
    pub workspace_for_new_window: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct MovePaneToNewTabResponse {
    pub tab_id: TabId,
    pub window_id: WindowId,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SpawnV2 {
    pub domain: config::keyassignment::SpawnTabDomain,
    /// If None, create a new window for this new tab
    pub window_id: Option<WindowId>,
    pub command: Option<CommandBuilder>,
    pub command_dir: Option<String>,
    pub size: TerminalSize,
    pub workspace: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct PaneRemoved {
    pub pane_id: PaneId,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct KillPane {
    pub pane_id: PaneId,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SpawnResponse {
    pub tab_id: TabId,
    pub pane_id: PaneId,
    pub window_id: WindowId,
    pub size: TerminalSize,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct WriteToPane {
    pub pane_id: PaneId,
    pub data: Vec<u8>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SendPaste {
    pub pane_id: PaneId,
    pub data: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SendKeyDown {
    pub pane_id: PaneId,
    pub event: termwiz::input::KeyEvent,
    pub input_serial: InputSerial,
}

/// InputSerial is used to sequence input requests with output events.
/// It started life as a monotonic sequence number but evolved into
/// the number of milliseconds since the unix epoch.
#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
pub struct InputSerial(u64);

impl InputSerial {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub fn now() -> Self {
        std::time::SystemTime::now().into()
    }

    pub fn elapsed_millis(&self) -> u64 {
        let now = InputSerial::now();
        now.0 - self.0
    }
}

impl From<std::time::SystemTime> for InputSerial {
    fn from(val: std::time::SystemTime) -> Self {
        let duration = val
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("SystemTime before unix epoch?");
        let millis: u64 = duration
            .as_millis()
            .try_into()
            .expect("millisecond count to fit in u64");
        InputSerial(millis)
    }
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SendMouseEvent {
    pub pane_id: PaneId,
    pub event: wezterm_term::input::MouseEvent,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SetClipboard {
    pub pane_id: PaneId,
    pub clipboard: Option<String>,
    pub selection: ClipboardSelection,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SetWindowWorkspace {
    pub window_id: WindowId,
    pub workspace: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct RenameWorkspace {
    pub old_workspace: String,
    pub new_workspace: String,
}

/// This is used both as a notification from server->client
/// and as a configuration request from client->server when
/// the client's preferred configuration changes
#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SetPalette {
    pub pane_id: PaneId,
    pub palette: ColorPalette,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct NotifyAlert {
    pub pane_id: PaneId,
    pub alert: Alert,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct TabAddedToWindow {
    pub tab_id: TabId,
    pub window_id: WindowId,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct TabResized {
    pub tab_id: TabId,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct TabTitleChanged {
    pub tab_id: TabId,
    pub title: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct WindowTitleChanged {
    pub window_id: WindowId,
    pub title: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct PaneFocused {
    pub pane_id: PaneId,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct WindowWorkspaceChanged {
    pub window_id: WindowId,
    pub workspace: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SetClientId {
    pub client_id: ClientId,
    pub is_proxy: bool,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SetFocusedPane {
    pub pane_id: PaneId,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct NotificationRecordState {
    pub notification_id: String,
    pub workspace: String,
    pub window_id: Option<WindowId>,
    pub tab_id: Option<TabId>,
    pub pane_id: Option<PaneId>,
    pub kind: String,
    pub title: String,
    pub body: Option<String>,
    pub unread: bool,
    pub unread_mode: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct CreateNotification {
    pub workspace: String,
    pub window_id: Option<WindowId>,
    pub tab_id: Option<TabId>,
    pub pane_id: Option<PaneId>,
    pub kind: String,
    pub title: String,
    pub body: Option<String>,
    pub unread_mode: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct CreateNotificationResponse {
    pub notification: NotificationRecordState,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ListNotifications {
    pub workspace: Option<String>,
    pub window_id: Option<WindowId>,
    pub tab_id: Option<TabId>,
    pub pane_id: Option<PaneId>,
    pub unread_only: bool,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ListNotificationsResponse {
    pub notifications: Vec<NotificationRecordState>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ClearNotifications {
    pub notification_ids: Vec<String>,
    pub workspace: Option<String>,
    pub tab_id: Option<TabId>,
    pub pane_id: Option<PaneId>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ClearNotificationsResponse {
    pub cleared_count: usize,
    pub notification_ids: Vec<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct MarkNotificationsRead {
    pub notification_ids: Vec<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct MarkNotificationsUnread {
    pub notification_ids: Vec<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct MarkNotificationsResponse {
    pub updated_count: usize,
    pub notification_ids: Vec<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct JumpUnread {
    pub pane_id: PaneId,
    pub direction: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct JumpUnreadResponse {
    pub pane_id: Option<PaneId>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct IdentifyNotificationTarget {
    pub pane_id: PaneId,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct IdentifyNotificationTargetResponse {
    pub workspace: String,
    pub window_id: WindowId,
    pub tab_id: TabId,
    pub pane_id: PaneId,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct GetNotificationCapabilities {}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct GetNotificationCapabilitiesResponse {
    pub notification_commands: Vec<String>,
    pub unread_modes: Vec<String>,
    pub supports_tabbar_markers: bool,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct WorkspaceStatusState {
    pub workspace: String,
    pub status: String,
    pub updated_at: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct WorkspaceProgressState {
    pub workspace: String,
    pub value: u8,
    pub updated_at: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct WorkspaceLogState {
    pub workspace: String,
    pub seq: u64,
    pub message: String,
    pub created_at: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct TaskPaneState {
    pub pane_id: PaneId,
    pub workspace: Option<String>,
    pub window_id: Option<WindowId>,
    pub tab_id: Option<TabId>,
    pub remain_on_exit: bool,
    pub silenced: bool,
    pub is_dead: bool,
    pub is_failed: bool,
    pub rerun_available: bool,
    pub tee_path: Option<String>,
    pub current_working_dir: Option<String>,
    pub updated_at: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct SetWorkspaceStatus {
    pub workspace: String,
    pub status: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct SetWorkspaceStatusResponse {
    pub status: WorkspaceStatusState,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ClearWorkspaceStatus {
    pub workspace: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ClearWorkspaceStatusResponse {
    pub cleared_count: usize,
    pub workspaces: Vec<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ListWorkspaceStatus {
    pub workspace: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ListWorkspaceStatusResponse {
    pub statuses: Vec<WorkspaceStatusState>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct SetWorkspaceProgress {
    pub workspace: String,
    pub value: u8,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct SetWorkspaceProgressResponse {
    pub progress: WorkspaceProgressState,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ClearWorkspaceProgress {
    pub workspace: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ClearWorkspaceProgressResponse {
    pub cleared_count: usize,
    pub workspaces: Vec<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct AppendWorkspaceLog {
    pub workspace: String,
    pub message: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct AppendWorkspaceLogResponse {
    pub entry: WorkspaceLogState,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ClearWorkspaceLog {
    pub workspace: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ClearWorkspaceLogResponse {
    pub cleared_count: usize,
    pub workspaces: Vec<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ListWorkspaceLog {
    pub workspace: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ListWorkspaceLogResponse {
    pub entries: Vec<WorkspaceLogState>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ListTaskPanes {
    pub pane_id: Option<PaneId>,
    pub workspace: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ListTaskPanesResponse {
    pub task_panes: Vec<TaskPaneState>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct SetRemainOnExit {
    pub pane_id: PaneId,
    pub remain_on_exit: bool,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct SetRemainOnExitResponse {
    pub pane: TaskPaneState,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct RerunPane {
    pub pane_id: PaneId,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct RerunPaneResponse {
    pub pane_id: PaneId,
    pub spawned_pane_id: PaneId,
    pub status: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct RespawnPane {
    pub pane_id: PaneId,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct RespawnPaneResponse {
    pub pane_id: PaneId,
    pub spawned_pane_id: PaneId,
    pub status: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct SilenceWatchdog {
    pub pane_id: PaneId,
    pub silenced: bool,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct SilenceWatchdogResponse {
    pub pane_id: PaneId,
    pub silenced: bool,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct PipePane {
    pub pane_id: PaneId,
    pub file_path: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct PipePaneResponse {
    pub pane_id: PaneId,
    pub tee_path: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct TaskPaneChanged {
    pub pane_id: PaneId,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct GetClientList;

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct GetClientListResponse {
    pub clients: Vec<ClientInfo>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Resize {
    pub containing_tab_id: TabId,
    pub pane_id: PaneId,
    pub size: TerminalSize,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SetPaneZoomed {
    pub containing_tab_id: TabId,
    pub pane_id: PaneId,
    pub zoomed: bool,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct GetPaneDirection {
    pub pane_id: PaneId,
    pub direction: PaneDirection,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct AdjustPaneSize {
    pub pane_id: PaneId,
    pub direction: PaneDirection,
    pub amount: usize,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct GetPaneDirectionResponse {
    pub pane_id: Option<PaneId>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ActivatePaneDirection {
    pub pane_id: PaneId,
    pub direction: PaneDirection,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct GetPaneRenderChanges {
    pub pane_id: PaneId,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct GetPaneRenderableDimensions {
    pub pane_id: PaneId,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct GetPaneRenderableDimensionsResponse {
    pub pane_id: PaneId,
    pub cursor_position: StableCursorPosition,
    pub dimensions: RenderableDimensions,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct LivenessResponse {
    pub pane_id: PaneId,
    pub is_alive: bool,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct GetPaneRenderChangesResponse {
    pub pane_id: PaneId,
    pub mouse_grabbed: bool,
    pub cursor_position: StableCursorPosition,
    pub dimensions: RenderableDimensions,
    pub dirty_lines: Vec<Range<StableRowIndex>>,
    pub title: String,
    pub working_dir: Option<SerdeUrl>,
    /// Lines that the server thought we'd almost certainly
    /// want to fetch as soon as we received this response
    pub bonus_lines: SerializedLines,

    pub input_serial: Option<InputSerial>,
    pub seqno: SequenceNo,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct GetLines {
    pub pane_id: PaneId,
    pub lines: Vec<Range<StableRowIndex>>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
struct CellCoordinates {
    line_idx: usize,
    cols: Range<usize>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
struct LineHyperlink {
    link: Hyperlink,
    coords: Vec<CellCoordinates>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SerializedImageCell {
    pub line_idx: StableRowIndex,
    pub cell_idx: usize,
    // The following fields are taken from termwiz::image::ImageCell
    pub top_left: TextureCoordinate,
    pub bottom_right: TextureCoordinate,
    /// Image::data::hash() for the ImageCell::data field
    pub data_hash: [u8; 32],
    pub z_index: i32,
    pub padding_left: u16,
    pub padding_top: u16,
    pub padding_right: u16,
    pub padding_bottom: u16,
    pub image_id: Option<u32>,
    pub placement_id: Option<u32>,
}

/// What's all this?
/// Cells hold references to Arc<Hyperlink> and it is important to us to
/// maintain identity of the hyperlinks in the individual cells, while also
/// only sending a single copy of the associated URL.
/// This section of code extracts the hyperlinks from the cells and builds
/// up a mapping that can be used to restore the identity when the `lines()`
/// method is called.
#[derive(Deserialize, Serialize, PartialEq, Debug, Default)]
pub struct SerializedLines {
    lines: Vec<(StableRowIndex, Line)>,
    hyperlinks: Vec<LineHyperlink>,
    images: Vec<SerializedImageCell>,
}

impl SerializedLines {
    /// Reconsitute hyperlinks or other attributes that were decomposed for
    /// serialization, and return the line data.
    pub fn extract_data(self) -> (Vec<(StableRowIndex, Line)>, Vec<SerializedImageCell>) {
        let lines = if self.hyperlinks.is_empty() {
            self.lines
        } else {
            let mut lines = self.lines;

            for link in self.hyperlinks {
                let url = Arc::new(link.link);

                for coord in link.coords {
                    if let Some((_, line)) = lines.get_mut(coord.line_idx) {
                        if let Some(cells) =
                            line.cells_mut_for_attr_changes_only().get_mut(coord.cols)
                        {
                            for cell in cells {
                                cell.attrs_mut().set_hyperlink(Some(Arc::clone(&url)));
                            }
                        }
                    }
                }
            }

            lines
        };
        (lines, self.images)
    }
}

impl From<Vec<(StableRowIndex, Line)>> for SerializedLines {
    fn from(mut lines: Vec<(StableRowIndex, Line)>) -> Self {
        let mut hyperlinks = vec![];
        let mut images = vec![];

        for (line_idx, (stable_row_idx, line)) in lines.iter_mut().enumerate() {
            let mut current_link: Option<Arc<Hyperlink>> = None;
            let mut current_range = 0..0;

            for (x, cell) in line
                .cells_mut_for_attr_changes_only()
                .iter_mut()
                .enumerate()
            {
                // Unset the hyperlink on the cell, if any, and record that
                // in the hyperlinks data for later restoration.
                if let Some(link) = cell.attrs_mut().hyperlink().map(Arc::clone) {
                    cell.attrs_mut().set_hyperlink(None);
                    match current_link.as_ref() {
                        Some(current) if Arc::ptr_eq(&current, &link) => {
                            // Continue the current streak
                            current_range = range_union(current_range, x..x + 1);
                        }
                        Some(prior) => {
                            // It's a different URL, push the current data and start a new one
                            hyperlinks.push(LineHyperlink {
                                link: (**prior).clone(),
                                coords: vec![CellCoordinates {
                                    line_idx,
                                    cols: current_range,
                                }],
                            });
                            current_range = x..x + 1;
                            current_link = Some(link);
                        }
                        None => {
                            // Starting a new streak
                            current_range = x..x + 1;
                            current_link = Some(link);
                        }
                    }
                } else if let Some(link) = current_link.take() {
                    // Wrap up a prior streak
                    hyperlinks.push(LineHyperlink {
                        link: (*link).clone(),
                        coords: vec![CellCoordinates {
                            line_idx,
                            cols: current_range,
                        }],
                    });
                    current_range = 0..0;
                }

                if let Some(cell_images) = cell.attrs().images() {
                    for imcell in cell_images {
                        let (padding_left, padding_top, padding_right, padding_bottom) =
                            imcell.padding();
                        images.push(SerializedImageCell {
                            line_idx: *stable_row_idx,
                            cell_idx: x,
                            top_left: imcell.top_left(),
                            bottom_right: imcell.bottom_right(),
                            z_index: imcell.z_index(),
                            padding_left,
                            padding_top,
                            padding_right,
                            padding_bottom,
                            image_id: imcell.image_id(),
                            placement_id: imcell.placement_id(),
                            data_hash: imcell.image_data().hash(),
                        });
                    }
                }
                cell.attrs_mut().clear_images();
            }
            if let Some(link) = current_link.take() {
                // Wrap up final streak
                hyperlinks.push(LineHyperlink {
                    link: (*link).clone(),
                    coords: vec![CellCoordinates {
                        line_idx,
                        cols: current_range,
                    }],
                });
            }
        }

        Self {
            lines,
            hyperlinks,
            images,
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct GetLinesResponse {
    pub pane_id: PaneId,
    pub lines: SerializedLines,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct EraseScrollbackRequest {
    pub pane_id: PaneId,
    pub erase_mode: ScrollbackEraseMode,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SearchScrollbackRequest {
    pub pane_id: PaneId,
    pub pattern: mux::pane::Pattern,
    pub range: Range<StableRowIndex>,
    pub limit: Option<u32>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SearchScrollbackResponse {
    pub results: Vec<mux::pane::SearchResult>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct GetImageCell {
    pub pane_id: PaneId,
    pub line_idx: StableRowIndex,
    pub cell_idx: usize,
    pub data_hash: [u8; 32],
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct GetImageCellResponse {
    pub pane_id: PaneId,
    pub data: Option<Arc<ImageData>>,
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::ser::{self, Impossible, SerializeStruct, Serializer};

    #[derive(Debug)]
    struct FieldNameCollectorError;

    impl std::fmt::Display for FieldNameCollectorError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("field name collector only supports struct serialization")
        }
    }

    impl std::error::Error for FieldNameCollectorError {}

    impl ser::Error for FieldNameCollectorError {
        fn custom<T>(_msg: T) -> Self
        where
            T: std::fmt::Display,
        {
            Self
        }
    }

    struct FieldNameCollector;

    struct FieldNameStructCollector {
        fields: Vec<String>,
    }

    impl SerializeStruct for FieldNameStructCollector {
        type Ok = Vec<String>;
        type Error = FieldNameCollectorError;

        fn serialize_field<T>(&mut self, key: &'static str, _value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
        {
            self.fields.push(key.to_string());
            Ok(())
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(self.fields)
        }
    }

    impl Serializer for FieldNameCollector {
        type Ok = Vec<String>;
        type Error = FieldNameCollectorError;
        type SerializeSeq = Impossible<Vec<String>, FieldNameCollectorError>;
        type SerializeTuple = Impossible<Vec<String>, FieldNameCollectorError>;
        type SerializeTupleStruct = Impossible<Vec<String>, FieldNameCollectorError>;
        type SerializeTupleVariant = Impossible<Vec<String>, FieldNameCollectorError>;
        type SerializeMap = Impossible<Vec<String>, FieldNameCollectorError>;
        type SerializeStruct = FieldNameStructCollector;
        type SerializeStructVariant = Impossible<Vec<String>, FieldNameCollectorError>;

        fn serialize_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStruct, Self::Error> {
            Ok(FieldNameStructCollector { fields: Vec::new() })
        }

        fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + Serialize,
        {
            Err(FieldNameCollectorError)
        }

        fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_unit_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
        ) -> Result<Self::Ok, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_newtype_struct<T>(
            self,
            _name: &'static str,
            _value: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + Serialize,
        {
            Err(FieldNameCollectorError)
        }

        fn serialize_newtype_variant<T>(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _value: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + Serialize,
        {
            Err(FieldNameCollectorError)
        }

        fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_tuple_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_tuple_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
            Err(FieldNameCollectorError)
        }

        fn serialize_struct_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
            Err(FieldNameCollectorError)
        }
    }

    fn collect_struct_fields<T: Serialize>(value: &T) -> Vec<String> {
        value.serialize(FieldNameCollector).expect("struct fields")
    }

    #[test]
    fn test_frame() {
        let mut encoded = Vec::new();
        encode_raw(0x81, 0x42, b"hello", false, &mut encoded).unwrap();
        assert_eq!(&encoded, b"\x08\x42\x81\x01hello");
        let decoded = decode_raw(encoded.as_slice()).unwrap();
        assert_eq!(decoded.ident, 0x81);
        assert_eq!(decoded.serial, 0x42);
        assert_eq!(decoded.data, b"hello");
    }

    #[test]
    fn test_frame_lengths() {
        let mut serial = 1;
        for target_len in &[128, 247, 256, 65536, 16777216] {
            let mut payload = Vec::with_capacity(*target_len);
            payload.resize(*target_len, b'a');
            let mut encoded = Vec::new();
            encode_raw(0x42, serial, payload.as_slice(), false, &mut encoded).unwrap();
            let decoded = decode_raw(encoded.as_slice()).unwrap();
            assert_eq!(decoded.ident, 0x42);
            assert_eq!(decoded.serial, serial);
            assert_eq!(decoded.data, payload);
            serial += 1;
        }
    }

    #[test]
    fn test_pdu_ping() {
        let mut encoded = Vec::new();
        Pdu::Ping(Ping {}).encode(&mut encoded, 0x40).unwrap();
        assert_eq!(&encoded, &[2, 0x40, 1]);
        assert_eq!(
            DecodedPdu {
                serial: 0x40,
                pdu: Pdu::Ping(Ping {})
            },
            Pdu::decode(encoded.as_slice()).unwrap()
        );
    }

    #[test]
    fn stream_decode() {
        let mut encoded = Vec::new();
        Pdu::Ping(Ping {}).encode(&mut encoded, 0x1).unwrap();
        Pdu::Pong(Pong {}).encode(&mut encoded, 0x2).unwrap();
        assert_eq!(encoded.len(), 6);

        let mut cursor = Cursor::new(encoded.as_slice());
        let mut read_buffer = Vec::new();

        assert_eq!(
            Pdu::try_read_and_decode(&mut cursor, &mut read_buffer).unwrap(),
            Some(DecodedPdu {
                serial: 1,
                pdu: Pdu::Ping(Ping {})
            })
        );
        assert_eq!(
            Pdu::try_read_and_decode(&mut cursor, &mut read_buffer).unwrap(),
            Some(DecodedPdu {
                serial: 2,
                pdu: Pdu::Pong(Pong {})
            })
        );
        let err = Pdu::try_read_and_decode(&mut cursor, &mut read_buffer).unwrap_err();
        assert_eq!(
            err.downcast_ref::<std::io::Error>().unwrap().kind(),
            std::io::ErrorKind::UnexpectedEof
        );
    }

    #[test]
    fn test_pdu_ping_base91() {
        let mut encoded = Vec::new();
        {
            let mut encoder = base91::Base91Encoder::new(&mut encoded);
            Pdu::Ping(Ping {}).encode(&mut encoder, 0x41).unwrap();
        }
        assert_eq!(&encoded, &[60, 67, 75, 65]);
        let decoded = base91::decode(&encoded);
        assert_eq!(
            DecodedPdu {
                serial: 0x41,
                pdu: Pdu::Ping(Ping {})
            },
            Pdu::decode(decoded.as_slice()).unwrap()
        );
    }

    #[test]
    fn test_pdu_pong() {
        let mut encoded = Vec::new();
        Pdu::Pong(Pong {}).encode(&mut encoded, 0x42).unwrap();
        assert_eq!(&encoded, &[2, 0x42, 2]);
        assert_eq!(
            DecodedPdu {
                serial: 0x42,
                pdu: Pdu::Pong(Pong {})
            },
            Pdu::decode(encoded.as_slice()).unwrap()
        );
    }

    #[test]
    fn test_bogus_pdu() {
        let mut encoded = Vec::new();
        encode_raw(0xdeadbeef, 0x42, b"hello", false, &mut encoded).unwrap();
        assert_eq!(
            DecodedPdu {
                serial: 0x42,
                pdu: Pdu::Invalid { ident: 0xdeadbeef }
            },
            Pdu::decode(encoded.as_slice()).unwrap()
        );
    }

    #[test]
    fn notification_payloads_serialize_scope_fields_with_stable_names() {
        let notification = NotificationRecordState {
            notification_id: "notif-1".to_string(),
            workspace: "alpha".to_string(),
            window_id: Some(11),
            tab_id: Some(TabId::new(21)),
            pane_id: None,
            kind: "task".to_string(),
            title: "Build failed".to_string(),
            body: Some("See logs".to_string()),
            unread: true,
            unread_mode: "sticky".to_string(),
            created_at: "2026-03-26T00:00:00Z".to_string(),
            updated_at: "2026-03-26T00:00:00Z".to_string(),
        };
        let create = CreateNotification {
            workspace: "alpha".to_string(),
            window_id: Some(11),
            tab_id: Some(TabId::new(21)),
            pane_id: None,
            kind: "task".to_string(),
            title: "Build failed".to_string(),
            body: Some("See logs".to_string()),
            unread_mode: "sticky".to_string(),
        };
        assert_eq!(notification.notification_id, "notif-1".to_string());
        assert_eq!(notification.workspace, "alpha".to_string());
        assert_eq!(notification.window_id, Some(11));
        assert_eq!(notification.tab_id, Some(TabId::new(21)));
        assert_eq!(notification.pane_id, None);
        assert_eq!(notification.kind, "task".to_string());
        assert_eq!(notification.title, "Build failed".to_string());
        assert_eq!(notification.body, Some("See logs".to_string()));
        assert!(notification.unread);
        assert_eq!(notification.unread_mode, "sticky".to_string());
        assert_eq!(notification.created_at, "2026-03-26T00:00:00Z".to_string());
        assert_eq!(notification.updated_at, "2026-03-26T00:00:00Z".to_string());

        assert_eq!(create.workspace, "alpha".to_string());
        assert_eq!(create.window_id, Some(11));
        assert_eq!(create.tab_id, Some(TabId::new(21)));
        assert_eq!(create.pane_id, None);
        assert_eq!(create.kind, "task".to_string());
        assert_eq!(create.title, "Build failed".to_string());
        assert_eq!(create.body, Some("See logs".to_string()));
        assert_eq!(create.unread_mode, "sticky".to_string());
    }

    #[test]
    fn notification_pdus_round_trip_through_codec() {
        fn assert_round_trip(pdu: Pdu) {
            let mut encoded = Vec::new();
            pdu.encode(&mut encoded, 77).unwrap();
            let decoded = Pdu::decode(encoded.as_slice()).unwrap();
            assert_eq!(decoded.serial, 77);
            assert_eq!(decoded.pdu, pdu);
        }

        let notification = NotificationRecordState {
            notification_id: "notif-1".to_string(),
            workspace: "alpha".to_string(),
            window_id: Some(11),
            tab_id: Some(TabId::new(21)),
            pane_id: Some(PaneId::new(31)),
            kind: "task".to_string(),
            title: "Build failed".to_string(),
            body: Some("See logs".to_string()),
            unread: true,
            unread_mode: "clear-on-focus".to_string(),
            created_at: "2026-03-26T00:00:00Z".to_string(),
            updated_at: "2026-03-26T00:00:01Z".to_string(),
        };

        assert_round_trip(Pdu::CreateNotification(CreateNotification {
            workspace: "alpha".to_string(),
            window_id: Some(11),
            tab_id: Some(TabId::new(21)),
            pane_id: Some(PaneId::new(31)),
            kind: "task".to_string(),
            title: "Build failed".to_string(),
            body: Some("See logs".to_string()),
            unread_mode: "clear-on-focus".to_string(),
        }));
        assert_round_trip(Pdu::CreateNotificationResponse(
            CreateNotificationResponse {
                notification: notification.clone(),
            },
        ));
        assert_round_trip(Pdu::ListNotifications(ListNotifications {
            workspace: Some("alpha".to_string()),
            window_id: Some(11),
            tab_id: Some(TabId::new(21)),
            pane_id: Some(PaneId::new(31)),
            unread_only: true,
        }));
        assert_round_trip(Pdu::ListNotificationsResponse(ListNotificationsResponse {
            notifications: vec![notification.clone()],
        }));
        assert_round_trip(Pdu::ClearNotifications(ClearNotifications {
            notification_ids: vec!["notif-1".to_string()],
            workspace: Some("alpha".to_string()),
            tab_id: Some(TabId::new(21)),
            pane_id: Some(PaneId::new(31)),
        }));
        assert_round_trip(Pdu::ClearNotificationsResponse(
            ClearNotificationsResponse {
                cleared_count: 1,
                notification_ids: vec!["notif-1".to_string()],
            },
        ));
        assert_round_trip(Pdu::MarkNotificationsRead(MarkNotificationsRead {
            notification_ids: vec!["notif-1".to_string()],
        }));
        assert_round_trip(Pdu::MarkNotificationsUnread(MarkNotificationsUnread {
            notification_ids: vec!["notif-1".to_string()],
        }));
        assert_round_trip(Pdu::MarkNotificationsResponse(MarkNotificationsResponse {
            updated_count: 1,
            notification_ids: vec!["notif-1".to_string()],
        }));
        assert_round_trip(Pdu::JumpUnread(JumpUnread {
            pane_id: PaneId::new(31),
            direction: "next".to_string(),
        }));
        assert_round_trip(Pdu::JumpUnreadResponse(JumpUnreadResponse {
            pane_id: Some(PaneId::new(32)),
        }));
        assert_round_trip(Pdu::IdentifyNotificationTarget(
            IdentifyNotificationTarget {
                pane_id: PaneId::new(31),
            },
        ));
        assert_round_trip(Pdu::IdentifyNotificationTargetResponse(
            IdentifyNotificationTargetResponse {
                workspace: "alpha".to_string(),
                window_id: 11,
                tab_id: TabId::new(21),
                pane_id: PaneId::new(31),
            },
        ));
        assert_round_trip(Pdu::GetNotificationCapabilities(
            GetNotificationCapabilities {},
        ));
        assert_round_trip(Pdu::GetNotificationCapabilitiesResponse(
            GetNotificationCapabilitiesResponse {
                notification_commands: vec!["notify".to_string()],
                unread_modes: vec!["clear-on-focus".to_string(), "sticky".to_string()],
                supports_tabbar_markers: true,
            },
        ));
    }

    #[test]
    fn workspace_metadata_payloads_serialize_stable_field_names() {
        let status = WorkspaceStatusState {
            workspace: "unity-main".to_string(),
            status: "running tests".to_string(),
            updated_at: "2026-03-27T00:00:00Z".to_string(),
        };
        let progress = WorkspaceProgressState {
            workspace: "unity-main".to_string(),
            value: 73,
            updated_at: "2026-03-27T00:00:05Z".to_string(),
        };
        let entry_one = WorkspaceLogState {
            workspace: "unity-main".to_string(),
            seq: 7,
            message: "build started".to_string(),
            created_at: "2026-03-27T00:00:10Z".to_string(),
        };
        let entry_two = WorkspaceLogState {
            workspace: "unity-main".to_string(),
            seq: 8,
            message: "build finished".to_string(),
            created_at: "2026-03-27T00:00:15Z".to_string(),
        };

        let progress_fields = collect_struct_fields(&progress);
        let log_fields = collect_struct_fields(&entry_one);
        let list_response = ListWorkspaceLogResponse {
            entries: vec![entry_one.clone(), entry_two.clone()],
        };

        assert_eq!(
            collect_struct_fields(&status),
            vec!["workspace", "status", "updated_at"]
        );
        assert_eq!(status.workspace, "unity-main".to_string());
        assert_eq!(status.status, "running tests".to_string());
        assert_eq!(status.updated_at, "2026-03-27T00:00:00Z".to_string());

        assert_eq!(progress_fields, vec!["workspace", "value", "updated_at"]);
        assert_eq!(progress.workspace, "unity-main".to_string());
        assert_eq!(progress.value, 73);
        assert_eq!(progress.updated_at, "2026-03-27T00:00:05Z".to_string());

        assert_eq!(
            log_fields,
            vec!["workspace", "seq", "message", "created_at"]
        );
        assert_eq!(entry_one.workspace, "unity-main".to_string());
        assert_eq!(entry_one.seq, 7);
        assert_eq!(entry_one.message, "build started".to_string());
        assert_eq!(entry_one.created_at, "2026-03-27T00:00:10Z".to_string());

        assert_eq!(collect_struct_fields(&list_response), vec!["entries"]);
        assert_eq!(list_response.entries, vec![entry_one, entry_two]);
    }

    #[test]
    fn workspace_metadata_pdus_round_trip_with_mutation_summaries() {
        fn assert_round_trip(pdu: Pdu) {
            let mut encoded = Vec::new();
            pdu.encode(&mut encoded, 88).unwrap();
            let decoded = Pdu::decode(encoded.as_slice()).unwrap();
            assert_eq!(decoded.serial, 88);
            assert_eq!(decoded.pdu, pdu);
        }

        let status = WorkspaceStatusState {
            workspace: "unity-main".to_string(),
            status: "idle".to_string(),
            updated_at: "2026-03-27T00:00:00Z".to_string(),
        };
        let progress = WorkspaceProgressState {
            workspace: "unity-main".to_string(),
            value: 42,
            updated_at: "2026-03-27T00:00:01Z".to_string(),
        };
        let entry = WorkspaceLogState {
            workspace: "unity-main".to_string(),
            seq: 12,
            message: "tests passed".to_string(),
            created_at: "2026-03-27T00:00:02Z".to_string(),
        };

        let clear_status = ClearWorkspaceStatusResponse {
            cleared_count: 2,
            workspaces: vec!["unity-main".to_string(), "unity-tools".to_string()],
        };
        let clear_progress = ClearWorkspaceProgressResponse {
            cleared_count: 1,
            workspaces: vec!["unity-main".to_string()],
        };
        let clear_log = ClearWorkspaceLogResponse {
            cleared_count: 1,
            workspaces: vec!["unity-main".to_string()],
        };

        assert_eq!(
            collect_struct_fields(&clear_status),
            vec!["cleared_count", "workspaces"]
        );
        assert_eq!(clear_status.cleared_count, 2);
        assert_eq!(
            clear_status.workspaces,
            vec!["unity-main".to_string(), "unity-tools".to_string()]
        );
        assert_eq!(
            collect_struct_fields(&clear_progress),
            vec!["cleared_count", "workspaces"]
        );
        assert_eq!(clear_progress.cleared_count, 1);
        assert_eq!(clear_progress.workspaces, vec!["unity-main".to_string()]);
        assert_eq!(
            collect_struct_fields(&clear_log),
            vec!["cleared_count", "workspaces"]
        );
        assert_eq!(clear_log.cleared_count, 1);
        assert_eq!(clear_log.workspaces, vec!["unity-main".to_string()]);

        assert_round_trip(Pdu::SetWorkspaceStatus(SetWorkspaceStatus {
            workspace: "unity-main".to_string(),
            status: "idle".to_string(),
        }));
        assert_round_trip(Pdu::SetWorkspaceStatusResponse(
            SetWorkspaceStatusResponse {
                status: status.clone(),
            },
        ));
        assert_round_trip(Pdu::ClearWorkspaceStatus(ClearWorkspaceStatus {
            workspace: Some("unity-main".to_string()),
        }));
        assert_round_trip(Pdu::ClearWorkspaceStatusResponse(clear_status));
        assert_round_trip(Pdu::ListWorkspaceStatus(ListWorkspaceStatus {
            workspace: None,
        }));
        assert_round_trip(Pdu::ListWorkspaceStatusResponse(
            ListWorkspaceStatusResponse {
                statuses: vec![status],
            },
        ));
        assert_round_trip(Pdu::SetWorkspaceProgress(SetWorkspaceProgress {
            workspace: "unity-main".to_string(),
            value: 42,
        }));
        assert_round_trip(Pdu::SetWorkspaceProgressResponse(
            SetWorkspaceProgressResponse { progress: progress },
        ));
        assert_round_trip(Pdu::ClearWorkspaceProgress(ClearWorkspaceProgress {
            workspace: None,
        }));
        assert_round_trip(Pdu::ClearWorkspaceProgressResponse(clear_progress));
        assert_round_trip(Pdu::ListWorkspaceProgress(ListWorkspaceProgress {
            workspace: Some("unity-main".to_string()),
        }));
        assert_round_trip(Pdu::ListWorkspaceProgressResponse(
            ListWorkspaceProgressResponse {
                progress: vec![WorkspaceProgressState {
                    workspace: "unity-main".to_string(),
                    value: 42,
                    updated_at: "2026-03-27T00:00:01Z".to_string(),
                }],
            },
        ));
        assert_round_trip(Pdu::AppendWorkspaceLog(AppendWorkspaceLog {
            workspace: "unity-main".to_string(),
            message: "tests passed".to_string(),
        }));
        assert_round_trip(Pdu::AppendWorkspaceLogResponse(
            AppendWorkspaceLogResponse {
                entry: entry.clone(),
            },
        ));
        assert_round_trip(Pdu::ClearWorkspaceLog(ClearWorkspaceLog {
            workspace: Some("unity-main".to_string()),
        }));
        assert_round_trip(Pdu::ClearWorkspaceLogResponse(clear_log));
        assert_round_trip(Pdu::ListWorkspaceLog(ListWorkspaceLog {
            workspace: Some("unity-main".to_string()),
            limit: Some(25),
        }));
        assert_round_trip(Pdu::ListWorkspaceLogResponse(ListWorkspaceLogResponse {
            entries: vec![entry],
        }));

        let task_pane = TaskPaneState {
            pane_id: PaneId::new(9),
            workspace: Some("unity-main".to_string()),
            window_id: Some(4),
            tab_id: Some(TabId::new(3)),
            remain_on_exit: true,
            silenced: false,
            is_dead: true,
            is_failed: true,
            rerun_available: true,
            tee_path: Some("/tmp/task.log".to_string()),
            current_working_dir: Some("file:///tmp/project".to_string()),
            updated_at: "2026-03-27T00:00:02Z".to_string(),
        };

        assert_eq!(
            collect_struct_fields(&task_pane),
            vec![
                "pane_id",
                "workspace",
                "window_id",
                "tab_id",
                "remain_on_exit",
                "silenced",
                "is_dead",
                "is_failed",
                "rerun_available",
                "tee_path",
                "current_working_dir",
                "updated_at",
            ]
        );

        let set_remain = SetRemainOnExitResponse {
            pane: task_pane.clone(),
        };
        let rerun = RerunPaneResponse {
            pane_id: PaneId::new(9),
            spawned_pane_id: PaneId::new(12),
            status: "rerun".to_string(),
        };
        let respawn = RespawnPaneResponse {
            pane_id: PaneId::new(9),
            spawned_pane_id: PaneId::new(13),
            status: "respawn".to_string(),
        };
        let silence = SilenceWatchdogResponse {
            pane_id: PaneId::new(9),
            silenced: true,
        };
        let pipe = PipePaneResponse {
            pane_id: PaneId::new(9),
            tee_path: Some("/tmp/task.log".to_string()),
        };

        assert_eq!(collect_struct_fields(&set_remain), vec!["pane"]);
        assert_eq!(
            collect_struct_fields(&rerun),
            vec!["pane_id", "spawned_pane_id", "status"]
        );
        assert_eq!(
            collect_struct_fields(&respawn),
            vec!["pane_id", "spawned_pane_id", "status"]
        );
        assert_eq!(collect_struct_fields(&silence), vec!["pane_id", "silenced"]);
        assert_eq!(collect_struct_fields(&pipe), vec!["pane_id", "tee_path"]);

        assert_round_trip(Pdu::ListTaskPanes(ListTaskPanes {
            pane_id: Some(PaneId::new(9)),
            workspace: Some("unity-main".to_string()),
        }));
        assert_round_trip(Pdu::ListTaskPanesResponse(ListTaskPanesResponse {
            task_panes: vec![task_pane.clone()],
        }));
        assert_round_trip(Pdu::SetRemainOnExit(SetRemainOnExit {
            pane_id: PaneId::new(9),
            remain_on_exit: true,
        }));
        assert_round_trip(Pdu::SetRemainOnExitResponse(set_remain));
        assert_round_trip(Pdu::RerunPane(RerunPane {
            pane_id: PaneId::new(9),
        }));
        assert_round_trip(Pdu::RerunPaneResponse(rerun));
        assert_round_trip(Pdu::RespawnPane(RespawnPane {
            pane_id: PaneId::new(9),
        }));
        assert_round_trip(Pdu::RespawnPaneResponse(respawn));
        assert_round_trip(Pdu::SilenceWatchdog(SilenceWatchdog {
            pane_id: PaneId::new(9),
            silenced: true,
        }));
        assert_round_trip(Pdu::SilenceWatchdogResponse(silence));
        assert_round_trip(Pdu::PipePane(PipePane {
            pane_id: PaneId::new(9),
            file_path: Some("/tmp/task.log".to_string()),
        }));
        assert_round_trip(Pdu::PipePaneResponse(pipe));
        assert_round_trip(Pdu::TaskPaneChanged(TaskPaneChanged {
            pane_id: PaneId::new(9),
        }));
    }
}
