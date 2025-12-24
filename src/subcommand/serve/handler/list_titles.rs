#[derive(askama::Template)]
#[template(path = "list_titles.html")]
pub struct ListTitlesResponse {
    pub(crate) page_titles: Vec<ListTitlesResponsePageTitle>,
}

impl axum::response::IntoResponse for ListTitlesResponse {
    fn into_response(self) -> axum::response::Response {
        let body = self.to_string();
        axum::response::Html(body).into_response()
    }
}

pub struct ListTitlesResponsePageTitle {
    pub(crate) page_ids: Vec<String>,
    pub(crate) value: String,
}

pub async fn handle(
    axum::extract::State(state): axum::extract::State<
        std::sync::Arc<std::sync::Mutex<crate::subcommand::serve::State>>,
    >,
) -> Result<ListTitlesResponse, axum::http::StatusCode> {
    let state = state.lock().map_err(|_| axum::http::StatusCode::CONFLICT)?;
    let page_titles = state
        .index
        .page_titles
        .iter()
        .map(|(title, page_ids)| ListTitlesResponsePageTitle {
            value: title.to_owned(),
            page_ids: page_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>(),
        })
        .collect::<Vec<ListTitlesResponsePageTitle>>();
    Ok(ListTitlesResponse { page_titles })
}
