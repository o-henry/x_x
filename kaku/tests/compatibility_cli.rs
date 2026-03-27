mod cli {
    use anyhow::anyhow;
    use clap::Parser;
    use std::ffi::OsString;

    #[derive(Debug, Parser, Clone, Copy, PartialEq, Eq)]
    pub enum CliOutputFormatKind {
        #[command(name = "table", about = "multi line space separated table")]
        Table,
        #[command(name = "json", about = "JSON format")]
        Json,
    }

    impl std::str::FromStr for CliOutputFormatKind {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "json" => Ok(Self::Json),
                "table" => Ok(Self::Table),
                _ => Err(anyhow!("unknown output format")),
            }
        }
    }

    pub fn resolve_relative_cwd(cwd: Option<OsString>) -> anyhow::Result<Option<String>> {
        match cwd {
            None => Ok(None),
            Some(cwd) => Ok(Some(
                std::env::current_dir()?
                    .join(cwd)
                    .to_str()
                    .ok_or_else(|| anyhow!("path is not representable as String"))?
                    .to_string(),
            )),
        }
    }

    pub mod list {
        include!("../src/cli/list.rs");

        impl ListCommand {
            pub fn format_for_test(&self) -> super::CliOutputFormatKind {
                self.format
            }
        }
    }

    pub mod list_task_panes {
        include!("../src/cli/list_task_panes.rs");

        impl ListTaskPanesCommand {
            pub fn request_for_test(&self) -> codec::ListTaskPanes {
                self.to_request()
            }

            pub fn render_json_for_test(
                task_panes: &[codec::TaskPaneState],
            ) -> anyhow::Result<String> {
                Self::render_json(task_panes)
            }
        }
    }

    pub mod spawn_command {
        include!("../src/cli/spawn_command.rs");

        #[derive(Debug)]
        pub struct SpawnDispatchShape {
            pub domain: config::keyassignment::SpawnTabDomain,
            pub window_id: Option<mux::window::WindowId>,
            pub command: Option<portable_pty::cmdbuilder::CommandBuilder>,
            pub command_dir: Option<String>,
            pub workspace: String,
        }

        impl SpawnCommand {
            pub fn new_for_test(
                domain_name: Option<&str>,
                new_window: bool,
                cwd: Option<&str>,
                workspace: Option<&str>,
                prog: &[&str],
            ) -> Self {
                Self {
                    pane_id: None,
                    domain_name: domain_name.map(str::to_string),
                    window_id: None,
                    new_window,
                    cwd: cwd.map(std::ffi::OsString::from),
                    workspace: workspace.map(str::to_string),
                    prog: prog.iter().map(std::ffi::OsString::from).collect(),
                }
            }

            pub fn dispatch_shape_for_test(
                &self,
                window_id: Option<mux::window::WindowId>,
                workspace: &str,
            ) -> anyhow::Result<SpawnDispatchShape> {
                Ok(SpawnDispatchShape {
                    domain: self.domain_name.clone().map_or(
                        config::keyassignment::SpawnTabDomain::DefaultDomain,
                        config::keyassignment::SpawnTabDomain::DomainName,
                    ),
                    window_id,
                    command: (!self.prog.is_empty()).then(|| {
                        portable_pty::cmdbuilder::CommandBuilder::from_argv(self.prog.clone())
                    }),
                    command_dir: super::resolve_relative_cwd(self.cwd.clone())?,
                    workspace: workspace.to_string(),
                })
            }
        }
    }

    pub mod split_pane {
        include!("../src/cli/split_pane.rs");

        #[derive(Debug)]
        pub struct SplitDispatchShape {
            pub pane_id: mux::pane::PaneId,
            pub direction: mux::tab::SplitDirection,
            pub target_is_second: bool,
            pub size: mux::tab::SplitSize,
            pub top_level: bool,
            pub command: Option<portable_pty::cmdbuilder::CommandBuilder>,
            pub command_dir: Option<String>,
            pub move_pane_id: Option<mux::pane::PaneId>,
        }

        impl SplitPane {
            pub fn new_for_test(direction: &str, percent: Option<u8>) -> Self {
                Self {
                    pane_id: Some(mux::pane::PaneId::new(9)),
                    horizontal: direction == "horizontal",
                    left: false,
                    right: direction == "right",
                    top: false,
                    bottom: direction != "horizontal" && direction != "right",
                    top_level: false,
                    cells: None,
                    percent,
                    cwd: None,
                    move_pane_id: None,
                    prog: Vec::new(),
                }
            }

            pub fn dispatch_shape_for_test(
                &self,
                pane_id: mux::pane::PaneId,
            ) -> anyhow::Result<SplitDispatchShape> {
                let direction = if self.left || self.right || self.horizontal {
                    mux::tab::SplitDirection::Horizontal
                } else if self.top || self.bottom {
                    mux::tab::SplitDirection::Vertical
                } else {
                    mux::tab::SplitDirection::Vertical
                };
                let target_is_second = !(self.left || self.top);
                let size = match (self.cells, self.percent) {
                    (Some(c), _) => mux::tab::SplitSize::Cells(c),
                    (_, Some(p)) => mux::tab::SplitSize::Percent(p),
                    (None, None) => mux::tab::SplitSize::Percent(50),
                };

                Ok(SplitDispatchShape {
                    pane_id,
                    direction,
                    target_is_second,
                    size,
                    top_level: self.top_level,
                    command: (!self.prog.is_empty()).then(|| {
                        portable_pty::cmdbuilder::CommandBuilder::from_argv(self.prog.clone())
                    }),
                    command_dir: super::resolve_relative_cwd(self.cwd.clone())?,
                    move_pane_id: self.move_pane_id,
                })
            }
        }
    }

    pub mod send_text {
        include!("../src/cli/send_text.rs");

        #[derive(Debug, PartialEq, Eq)]
        pub enum SendTextDispatch {
            Paste {
                pane_id: mux::pane::PaneId,
                data: String,
            },
            Write {
                pane_id: mux::pane::PaneId,
                data: Vec<u8>,
            },
        }

        impl SendText {
            pub fn dispatch_for_test(
                &self,
                pane_id: mux::pane::PaneId,
                stdin_text: Option<&str>,
            ) -> SendTextDispatch {
                let data = self
                    .text
                    .clone()
                    .or_else(|| stdin_text.map(str::to_string))
                    .expect("test text");

                if self.no_paste {
                    SendTextDispatch::Write {
                        pane_id,
                        data: data.into_bytes(),
                    }
                } else {
                    SendTextDispatch::Paste { pane_id, data }
                }
            }
        }
    }

    pub mod get_text {
        include!("../src/cli/get_text.rs");

        impl GetText {
            pub fn line_range_for_test(
                &self,
                dimensions: &mux::renderable::RenderableDimensions,
            ) -> (wezterm_term::StableRowIndex, wezterm_term::StableRowIndex) {
                let start_line = match self.start_line {
                    None => dimensions.physical_top,
                    Some(n) if n >= 0 => {
                        dimensions.physical_top + n as wezterm_term::StableRowIndex
                    }
                    Some(n) => {
                        let line = dimensions.physical_top as isize + n as isize;
                        if line < dimensions.scrollback_top as isize {
                            dimensions.scrollback_top
                        } else {
                            line as wezterm_term::StableRowIndex
                        }
                    }
                };

                let end_line = match self.end_line {
                    None => {
                        dimensions.physical_top
                            + dimensions.viewport_rows as wezterm_term::StableRowIndex
                    }
                    Some(n) if n >= 0 => {
                        dimensions.physical_top + n as wezterm_term::StableRowIndex
                    }
                    Some(n) => {
                        let line = dimensions.physical_top as isize + n as isize;
                        if line < dimensions.scrollback_top as isize {
                            dimensions.scrollback_top
                        } else {
                            line as wezterm_term::StableRowIndex
                        }
                    }
                };

                (start_line, end_line)
            }
        }
    }

    pub mod set_tab_title {
        include!("../src/cli/set_tab_title.rs");

        impl SetTabTitle {
            pub fn request_for_test(&self, tab_id: mux::tab::TabId) -> codec::TabTitleChanged {
                codec::TabTitleChanged {
                    tab_id,
                    title: self.title.clone(),
                }
            }
        }
    }
}

use clap::Parser;
use codec::TaskPaneState;
use config::keyassignment::SpawnTabDomain;
use mux::pane::PaneId;
use mux::renderable::RenderableDimensions;
use mux::tab::{SplitDirection, SplitSize, TabId};
use serde_json::Value;
use wezterm_term::StableRowIndex;

use cli::get_text::GetText;
use cli::list::ListCommand;
use cli::list_task_panes::ListTaskPanesCommand;
use cli::send_text::{SendText, SendTextDispatch};
use cli::set_tab_title::SetTabTitle;
use cli::spawn_command::SpawnCommand;
use cli::split_pane::SplitPane;
use cli::CliOutputFormatKind;

fn sample_task_pane() -> TaskPaneState {
    TaskPaneState {
        pane_id: PaneId::new(4),
        workspace: Some("unity-main".to_string()),
        window_id: Some(1),
        tab_id: Some(TabId::new(2)),
        remain_on_exit: true,
        silenced: false,
        is_dead: true,
        is_failed: true,
        rerun_available: true,
        tee_path: Some("/tmp/task.log".to_string()),
        current_working_dir: Some("/tmp/project".to_string()),
        updated_at: "2026-03-27T00:00:00Z".to_string(),
    }
}

fn dims(physical_top: StableRowIndex, scrollback_top: StableRowIndex) -> RenderableDimensions {
    RenderableDimensions {
        cols: 80,
        viewport_rows: 24,
        scrollback_rows: 200,
        physical_top,
        scrollback_top,
        dpi: 96,
        pixel_width: 800,
        pixel_height: 480,
        reverse_video: false,
    }
}

#[test]
fn compatibility_cli_list_and_list_task_panes_keep_independent_format_contracts() {
    let list_cmd = ListCommand::parse_from(["kaku", "--format", "json"]);
    assert!(matches!(
        list_cmd.format_for_test(),
        CliOutputFormatKind::Json
    ));

    let lifecycle_cmd = ListTaskPanesCommand::parse_from([
        "kaku",
        "--pane-id",
        "4",
        "--workspace",
        "unity-main",
        "--format",
        "json",
    ]);
    let request = lifecycle_cmd.request_for_test();
    assert_eq!(request.pane_id, Some(PaneId::new(4)));
    assert_eq!(request.workspace.as_deref(), Some("unity-main"));

    let json = ListTaskPanesCommand::render_json_for_test(&[sample_task_pane()]).expect("json");
    let payload: Value = serde_json::from_str(&json).expect("json payload");
    let keys = payload.as_array().expect("array")[0]
        .as_object()
        .expect("object")
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    assert!(keys.contains(&"remain_on_exit".to_string()));
    assert!(keys.contains(&"rerun_available".to_string()));
}

#[test]
fn compatibility_cli_spawn_preserves_workspace_cwd_and_command_shape() {
    let cmd = SpawnCommand::new_for_test(
        None,
        true,
        Some("."),
        Some("unity-main"),
        &["/bin/echo", "hello"],
    );

    let dispatch = cmd
        .dispatch_shape_for_test(None, "unity-main")
        .expect("dispatch shape");

    assert!(matches!(dispatch.domain, SpawnTabDomain::DefaultDomain));
    assert_eq!(dispatch.window_id, None);
    assert_eq!(dispatch.workspace, "unity-main");
    assert_eq!(
        dispatch.command_dir,
        Some(
            std::env::current_dir()
                .expect("cwd")
                .join(".")
                .display()
                .to_string()
        )
    );
    let builder = dispatch.command.expect("command builder");
    assert_eq!(
        builder.get_argv(),
        &[
            std::ffi::OsString::from("/bin/echo"),
            std::ffi::OsString::from("hello")
        ]
    );
}

#[test]
fn compatibility_cli_split_pane_keeps_direction_and_size_dispatch_defaults() {
    let cmd = SplitPane::new_for_test("right", Some(30));
    let dispatch = cmd
        .dispatch_shape_for_test(PaneId::new(9))
        .expect("dispatch shape");

    assert_eq!(dispatch.pane_id, PaneId::new(9));
    assert_eq!(dispatch.direction, SplitDirection::Horizontal);
    assert!(dispatch.target_is_second);
    assert_eq!(dispatch.size, SplitSize::Percent(30));
    assert!(!dispatch.top_level);
    assert!(dispatch.command.is_none());
    assert!(dispatch.command_dir.is_none());
    assert_eq!(dispatch.move_pane_id, None);
}

#[test]
fn compatibility_cli_send_text_preserves_paste_and_raw_write_modes() {
    let paste = SendText::parse_from(["kaku", "--pane-id", "2", "hello"]);
    assert_eq!(
        paste.dispatch_for_test(PaneId::new(2), None),
        SendTextDispatch::Paste {
            pane_id: PaneId::new(2),
            data: "hello".to_string(),
        }
    );

    let raw = SendText::parse_from(["kaku", "--pane-id", "2", "--no-paste", "hello"]);
    assert_eq!(
        raw.dispatch_for_test(PaneId::new(2), None),
        SendTextDispatch::Write {
            pane_id: PaneId::new(2),
            data: b"hello".to_vec(),
        }
    );
}

#[test]
fn compatibility_cli_get_text_keeps_relative_line_window_behavior() {
    let cmd = GetText::parse_from([
        "kaku",
        "--pane-id",
        "4",
        "--start-line",
        "-5",
        "--end-line",
        "3",
    ]);

    let (start, end) = cmd.line_range_for_test(&dims(20, 8));
    assert_eq!(start, 15);
    assert_eq!(end, 23);
}

#[test]
fn compatibility_cli_set_tab_title_keeps_explicit_target_request_shape() {
    let cmd = SetTabTitle::parse_from(["kaku", "--tab-id", "7", "Build"]);
    let request = cmd.request_for_test(TabId::new(7));
    assert_eq!(request.tab_id, TabId::new(7));
    assert_eq!(request.title, "Build");
}
