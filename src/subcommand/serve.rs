mod handler;
mod index;

use anyhow::Context as _;

struct State {
    config: crate::config::Config,
    index: self::index::Index,
}

pub(super) async fn execute() -> anyhow::Result<()> {
    let config = crate::config::Config::load().await?;

    // create index
    let index = self::index::Index::new(config.clone())?;

    let port = config.port();

    let watch_dir = config.data_dir().to_path_buf();
    let state = std::sync::Arc::new(std::sync::Mutex::new(State { config, index }));

    // run watcher
    fn update_page_meta(
        state: std::sync::Arc<std::sync::Mutex<State>>,
        path: &std::path::Path,
    ) -> anyhow::Result<()> {
        let mut state = state.lock().map_err(|_| anyhow::anyhow!("locking state"))?;

        let page_id = crate::page_io::PageIo::page_id(path)?;

        if !path.exists() {
            state.index.remove(&page_id);
            return Ok(());
        }

        state.index.update(&page_id)?;

        Ok(())
    }

    async fn new_watcher(
        state_for_watcher: std::sync::Arc<std::sync::Mutex<State>>,
        watch_dir: std::path::PathBuf,
    ) -> anyhow::Result<()> {
        let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
        let mut watcher = notify::recommended_watcher(tx).context("create watcher")?;
        notify::Watcher::watch(&mut watcher, &watch_dir, notify::RecursiveMode::Recursive)
            .context("watch dir")?;
        for res in rx {
            match res {
                Ok(event) => {
                    match event.kind {
                        notify::EventKind::Any
                        | notify::EventKind::Access(_)
                        | notify::EventKind::Other => {
                            // do nothing
                        }
                        notify::EventKind::Create(_)
                        | notify::EventKind::Modify(_)
                        | notify::EventKind::Remove(_) => {
                            for path in event.paths {
                                update_page_meta(state_for_watcher.clone(), &path)
                                    .context("update page meta")?;
                            }
                        }
                    }
                }
                Err(e) => anyhow::bail!("watch error: {:?}", e),
            }
        }
        Ok(())
    }
    tokio::spawn(new_watcher(state.clone(), watch_dir));

    let router = axum::Router::new()
        .route(
            "/",
            axum::routing::get(self::handler::get_root_or_list_pages),
        )
        .route("/{id}", axum::routing::get(self::handler::get))
        .route("/pages", axum::routing::get(self::handler::list))
        .route("/pages/{id}", axum::routing::get(self::handler::get))
        .route(
            "/images/{image_name}",
            axum::routing::get(self::handler::get_image),
        )
        .route(
            "/scripts/index.js",
            axum::routing::get(self::handler::get_script_index),
        )
        .route(
            "/styles/index.css",
            axum::routing::get(self::handler::get_style_index),
        )
        .route("/titles", axum::routing::get(self::handler::list_titles))
        .route(
            "/titles/{title}",
            axum::routing::get(self::handler::get_page_by_title),
        )
        .with_state(state);

    let ip_addr = <std::net::IpAddr as std::str::FromStr>::from_str("127.0.0.1")
        .expect("127.0.0.1 to be valid as IpAddr");
    let listener = tokio::net::TcpListener::bind((ip_addr, port)).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
