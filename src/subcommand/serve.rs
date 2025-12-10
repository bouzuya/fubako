mod handler;

use anyhow::Context as _;

struct State {
    backlinks: std::collections::BTreeMap<
        crate::page_id::PageId,
        std::collections::BTreeSet<crate::page_id::PageId>,
    >,
    config: crate::config::Config,
    page_metas: std::collections::BTreeMap<crate::page_id::PageId, crate::page_meta::PageMeta>,
}

pub(super) async fn execute() -> anyhow::Result<()> {
    let config = crate::config::Config::load().await?;

    // create index
    let page_ids = crate::page_io::PageIo::read_page_ids(&config)?;

    let mut page_metas = std::collections::BTreeMap::new();
    for page_id in &page_ids {
        let page_meta = crate::page_io::PageIo::read_page_meta(&config, page_id)?;
        page_metas.insert(page_id.clone(), page_meta);
    }

    let mut backlinks = std::collections::BTreeMap::new();
    for (page_id, page_meta) in &page_metas {
        for linked_page_id in &page_meta.links {
            backlinks
                .entry(linked_page_id.clone())
                .or_insert_with(std::collections::BTreeSet::new)
                .insert(page_id.clone());
        }
    }

    let watch_dir = config.data_dir.clone();
    let state = std::sync::Arc::new(std::sync::Mutex::new(State {
        backlinks,
        config,
        page_metas,
    }));

    // run watcher
    fn update_page_meta(
        state: std::sync::Arc<std::sync::Mutex<State>>,
        path: &std::path::Path,
    ) -> anyhow::Result<()> {
        let mut state = state.lock().map_err(|_| anyhow::anyhow!("locking state"))?;

        let page_id = crate::page_io::PageIo::page_id(path)?;

        if !path.exists() {
            let old_page_meta = state.page_metas.get(&page_id).cloned();
            match old_page_meta {
                Some(old_page_meta) => {
                    // remove old links from backlinks
                    for linked_page_id in &old_page_meta.links {
                        if let Some(set) = state.backlinks.get_mut(linked_page_id) {
                            set.remove(&page_id);
                        }
                    }
                }
                None => {
                    // do nothing
                }
            }
            return Ok(());
        }

        let new_page_meta = crate::page_io::PageIo::read_page_meta(&state.config, &page_id)?;

        let old_page_meta = state.page_metas.get(&page_id).cloned();
        match old_page_meta {
            Some(old_page_meta) => {
                // remove old links from backlinks
                for linked_page_id in &old_page_meta.links {
                    if let Some(set) = state.backlinks.get_mut(linked_page_id) {
                        set.remove(&page_id);
                    }
                }
            }
            None => {
                // do nothing
            }
        }

        for linked_page_id in &new_page_meta.links {
            state
                .backlinks
                .entry(linked_page_id.clone())
                .or_insert_with(std::collections::BTreeSet::new)
                .insert(page_id.clone());
        }

        state
            .page_metas
            .insert(page_id.clone(), new_page_meta.clone());

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
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, router).await?;
    Ok(())
}
