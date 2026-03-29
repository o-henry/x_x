use kaku_native_shell::runtime_bridge::{NativeShellBootstrapPlan, RuntimeOwnershipProof};
use wezterm_gui_subcommands::DEFAULT_WINDOW_CLASS;

#[test]
fn runtime_bootstrap_ownership() {
    let plan = NativeShellBootstrapPlan::default();
    let proof = RuntimeOwnershipProof::default();
    assert_eq!(plan.window_class, DEFAULT_WINDOW_CLASS);
    assert!(plan.should_publish);
    assert_eq!(plan.default_workspace.as_deref(), Some("default"));
    assert!(plan.proves_socket_authority());
    assert_eq!(proof.primary_shell, "kaku-native-shell");
    assert_eq!(proof.fallback_shell, "kaku-gui");
    assert_eq!(proof.socket_env_var, "KAKU_UNIX_SOCKET");
    assert_eq!(proof.socket_file_prefix, "gui-sock-");
}
