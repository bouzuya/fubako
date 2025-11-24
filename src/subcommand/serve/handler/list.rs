#[derive(askama::Template)]
#[template(path = "list.html")]
pub struct ListResponse {
    page_metas: Vec<ListResponsePageMeta>,
    q: String,
}

impl axum::response::IntoResponse for ListResponse {
    fn into_response(self) -> axum::response::Response {
        let body = self.to_string();
        axum::response::Html(body).into_response()
    }
}

pub struct ListResponsePageMeta {
    id: String,
    title: String,
}

#[derive(serde::Deserialize)]
pub struct ListRequestQuery {
    q: Option<String>,
}

pub async fn handle(
    axum::extract::State(state): axum::extract::State<
        std::sync::Arc<std::sync::Mutex<crate::subcommand::serve::State>>,
    >,
    axum::extract::Query(ListRequestQuery { q }): axum::extract::Query<ListRequestQuery>,
) -> Result<ListResponse, axum::http::StatusCode> {
    let q = q.unwrap_or_default().trim().to_owned();
    let state = state.lock().map_err(|_| axum::http::StatusCode::CONFLICT)?;
    let config = &state.config;
    let page_metas = state
        .page_metas
        .iter()
        .filter(|(page_id, _page_meta)| {
            q.is_empty() || {
                crate::page_io::PageIo::read_page_content(config, page_id)
                    .is_ok_and(|content| content.contains(&q))
            }
        })
        .map(|(id, meta)| ListResponsePageMeta {
            id: id.to_string(),
            title: meta.title.clone().unwrap_or_default(),
        })
        .collect::<Vec<ListResponsePageMeta>>();
    Ok(ListResponse { page_metas, q })
}
