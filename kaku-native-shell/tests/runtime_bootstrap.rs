use kaku_native_shell::runtime_bridge::NativeShellBootstrapPlan;
use wezterm_gui_subcommands::DEFAULT_WINDOW_CLASS;

#[test]
fn runtime_bootstrap_ownership() {
    let plan = NativeShellBootstrapPlan::default();
    assert_eq!(plan.window_class, DEFAULT_WINDOW_CLASS);
    assert!(plan.should_publish);
    assert_eq!(plan.default_workspace.as_deref(), Some("default"));
    assert!(plan.proves_socket_authority());
}
