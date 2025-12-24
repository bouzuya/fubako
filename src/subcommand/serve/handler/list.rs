#[derive(askama::Template)]
#[template(path = "list.html")]
pub struct ListResponse {
    pub(crate) page_metas: Vec<ListResponsePageMeta>,
    pub(crate) q: String,
}

impl axum::response::IntoResponse for ListResponse {
    fn into_response(self) -> axum::response::Response {
        let body = self.to_string();
        axum::response::Html(body).into_response()
    }
}

pub struct ListResponsePageMeta {
    pub(crate) id: String,
    pub(crate) title: String,
}

#[derive(serde::Deserialize)]
pub struct ListRequestQuery {
    pub(crate) q: Option<String>,
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
        .index
        .page_metas
        .iter()
        .filter(|(page_id, _page_meta)| {
            q.is_empty() || {
                crate::page_io::PageIo::read_page_content(config, page_id)
                    .is_ok_and(|content| match_content(&content, &q))
            }
        })
        .map(|(id, meta)| ListResponsePageMeta {
            id: id.to_string(),
            title: meta.title.clone().unwrap_or_default(),
        })
        .collect::<Vec<ListResponsePageMeta>>();
    Ok(ListResponse { page_metas, q })
}

fn match_content(content: &str, q: &str) -> bool {
    let content = content.to_lowercase();
    let q = q.to_lowercase();
    q.split_whitespace().all(|q| content.contains(q))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_content() {
        let content = "This is a sample page content.";
        // single keyword
        assert!(match_content(content, "This"));
        assert!(match_content(content, "page"));
        // ignore case
        assert!(match_content(content, "this"));
        // multiple keywords
        assert!(match_content(content, "This page"));
        assert!(match_content(content, "This\tpage"));
        assert!(match_content(content, "This\u{3000}page"));
        assert!(match_content(content, "This \t\u{3000}page"));
        // not found
        assert!(!match_content(content, "notfound"));
    }
}
