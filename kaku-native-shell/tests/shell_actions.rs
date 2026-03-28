use kaku_native_shell::app_controller::ShellAction;

#[test]
fn shell_actions_visible() {
    let actions = ShellAction::operator_actions();
    assert!(actions.contains(&ShellAction::Refresh));
    assert!(actions.contains(&ShellAction::LaunchTerminal));
    assert!(actions.contains(&ShellAction::SetStatus));
    assert!(actions.contains(&ShellAction::ClearStatus));
    assert!(actions.contains(&ShellAction::SetProgress));
    assert!(actions.contains(&ShellAction::ClearProgress));
    assert!(actions.contains(&ShellAction::AppendLog));
    assert!(actions.contains(&ShellAction::MarkVisibleRead));
}
