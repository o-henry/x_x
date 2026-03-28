use mux::Mux;
use std::path::PathBuf;
use std::sync::Arc;
use wezterm_gui_subcommands::DEFAULT_WINDOW_CLASS;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeShellBootstrapPlan {
    pub window_class: String,
    pub default_domain: Option<String>,
    pub default_workspace: Option<String>,
    pub should_publish: bool,
}

impl Default for NativeShellBootstrapPlan {
    fn default() -> Self {
        Self {
            window_class: DEFAULT_WINDOW_CLASS.to_string(),
            default_domain: None,
            default_workspace: Some("default".to_string()),
            should_publish: true,
        }
    }
}

impl NativeShellBootstrapPlan {
    pub fn proves_socket_authority(&self) -> bool {
        self.should_publish && self.default_workspace.as_deref() == Some("default")
    }
}

pub struct NativeShellBootstrapResult {
    pub mux: Arc<Mux>,
    pub unix_socket_path: PathBuf,
}

pub struct RuntimeOwnershipProof {
    pub primary_shell: &'static str,
    pub fallback_shell: &'static str,
    pub socket_env_var: &'static str,
    pub socket_file_prefix: &'static str,
}

impl RuntimeOwnershipProof {
    pub fn default() -> Self {
        Self {
            primary_shell: "kaku-native-shell",
            fallback_shell: "kaku-gui",
            socket_env_var: "KAKU_UNIX_SOCKET",
            socket_file_prefix: "gui-sock-",
        }
    }
}

pub fn bootstrap_native_shell_runtime(
    plan: &NativeShellBootstrapPlan,
) -> anyhow::Result<NativeShellBootstrapResult> {
    let result = kaku_runtime::bootstrap_native_shell_runtime(
        &plan.window_class,
        plan.default_domain.as_deref(),
        plan.default_workspace.as_deref(),
        plan.should_publish,
    )?;
    Ok(NativeShellBootstrapResult {
        mux: result.mux,
        unix_socket_path: result.unix_socket_path,
    })
}
