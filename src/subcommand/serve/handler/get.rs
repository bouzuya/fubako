#[derive(Debug, askama::Template)]
#[template(path = "get.html")]
pub struct GetResponse {
    backlinks: Vec<String>,
    html: String,
    id: String,
    title: String,
}

impl axum::response::IntoResponse for GetResponse {
    fn into_response(self) -> axum::response::Response {
        let body = self.to_string();
        axum::response::Html(body).into_response()
    }
}

pub async fn handle(
    axum::extract::State(state): axum::extract::State<
        std::sync::Arc<std::sync::Mutex<crate::subcommand::serve::State>>,
    >,
    axum::extract::Path(page_id): axum::extract::Path<crate::page_id::PageId>,
) -> Result<GetResponse, axum::http::StatusCode> {
    let state = state.lock().map_err(|_| axum::http::StatusCode::CONFLICT)?;
    let page_meta = state
        .page_metas
        .get(&page_id)
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;

    let html = crate::page_io::PageIo::read_page_content(&state.config, &page_id)
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)?;

    Ok(GetResponse {
        backlinks: state
            .backlinks
            .get(&page_id)
            .map(|set| set.iter().map(|id| id.to_string()).collect::<Vec<String>>())
            .unwrap_or_default(),
        html,
        id: page_id.to_string(),
        title: page_meta.title.clone().unwrap_or_default(),
    })
}
