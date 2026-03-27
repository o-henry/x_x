use anyhow::Context;
use config::ConfigHandle;
use mux::domain::{Domain, LocalDomain};
use mux::Mux;
use std::path::PathBuf;
use std::sync::Arc;
use wezterm_mux_server_impl::update_mux_domains;

pub fn prepare_gui_runtime_env() -> anyhow::Result<PathBuf> {
    let unix_socket_path =
        config::RUNTIME_DIR.join(format!("gui-sock-{}", unsafe { libc::getpid() }));
    std::env::set_var("KAKU_UNIX_SOCKET", &unix_socket_path);
    wezterm_blob_leases::register_storage(Arc::new(
        wezterm_blob_leases::simple_tempdir::SimpleTempDir::new_in(&*config::CACHE_DIR)?,
    ))?;
    Ok(unix_socket_path)
}

pub fn spawn_mux_server(
    unix_socket_path: PathBuf,
    should_publish: bool,
    window_class: &str,
) -> anyhow::Result<()> {
    let mut listener =
        wezterm_mux_server_impl::local::LocalListener::with_domain(&config::UnixDomain {
            socket_path: Some(unix_socket_path.clone()),
            ..Default::default()
        })?;
    let window_class = window_class.to_string();
    std::thread::spawn(move || {
        let name_holder;
        if should_publish {
            name_holder =
                wezterm_client::discovery::publish_gui_sock_path(&unix_socket_path, &window_class);
            if let Err(err) = &name_holder {
                log::warn!("{:#}", err);
            }
        }

        listener.run();
        std::fs::remove_file(unix_socket_path).ok();
    });

    Ok(())
}

pub fn setup_mux(
    local_domain: Arc<dyn Domain>,
    config: &ConfigHandle,
    default_domain_name: Option<&str>,
    default_workspace_name: Option<&str>,
) -> anyhow::Result<Arc<Mux>> {
    let mux = Arc::new(mux::Mux::new(Some(local_domain.clone())));
    Mux::set_mux(&mux);
    let client_id = Arc::new(mux::client::ClientId::new());
    mux.register_client(client_id.clone());
    mux.replace_identity(Some(client_id));
    let default_workspace_name = default_workspace_name.unwrap_or(
        config
            .default_workspace
            .as_deref()
            .unwrap_or(mux::DEFAULT_WORKSPACE),
    );
    mux.set_active_workspace(&default_workspace_name);

    let default_name =
        default_domain_name.unwrap_or(config.default_domain.as_deref().unwrap_or("local"));
    if default_name != "local" {
        update_mux_domains(config)?;
    }

    let domain = mux.get_domain_by_name(default_name).with_context(|| {
        format!(
            "desired default domain '{}' was not found in mux!?",
            default_name
        )
    })?;
    mux.set_default_domain(&domain);

    Ok(mux)
}

pub fn build_initial_mux(
    config: &ConfigHandle,
    default_domain_name: Option<&str>,
    default_workspace_name: Option<&str>,
) -> anyhow::Result<Arc<Mux>> {
    let domain: Arc<dyn Domain> = Arc::new(LocalDomain::new("local")?);
    setup_mux(domain, config, default_domain_name, default_workspace_name)
}
