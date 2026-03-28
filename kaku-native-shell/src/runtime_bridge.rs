use mux::Mux;
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

pub fn bootstrap_native_shell_runtime(
    plan: &NativeShellBootstrapPlan,
) -> anyhow::Result<Arc<Mux>> {
    kaku_runtime::bootstrap_gui_runtime(
        &plan.window_class,
        plan.default_domain.as_deref(),
        plan.default_workspace.as_deref(),
        plan.should_publish,
    )
}

