use kaku_native_shell::snapshot::{ShellLayoutContract, WorkspaceSummary};

#[test]
fn shell_layout_contract() {
    let layout = ShellLayoutContract::default();
    assert_eq!(layout.chrome_height, 46);
    assert_eq!(layout.rail_width, 176);
    assert!(layout.section_names().contains(&"chrome"));
    assert!(layout.section_names().contains(&"rail"));
    assert!(layout.section_names().contains(&"main"));
    assert!(layout.section_names().contains(&"context"));

    let workspace = WorkspaceSummary {
        name: "default".to_string(),
        unread_count: 1,
        running_count: 2,
        failed_count: 0,
        status: Some("running".to_string()),
        progress: Some(32),
        log_count: 5,
    };
    assert_eq!(workspace.name, "default");
    assert_eq!(workspace.progress, Some(32));
}
