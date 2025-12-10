pub enum GetRootOrListPagesResponse {
    Get(super::get::GetResponse),
    List,
}

impl From<super::get::GetResponse> for GetRootOrListPagesResponse {
    fn from(value: super::get::GetResponse) -> Self {
        Self::Get(value)
    }
}

impl axum::response::IntoResponse for GetRootOrListPagesResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Get(resp) => resp.into_response(),
            Self::List => {
                let mut response = axum::http::StatusCode::FOUND.into_response();
                response.headers_mut().insert(
                    axum::http::header::LOCATION,
                    axum::http::HeaderValue::from_static("/pages"),
                );
                response
            }
        }
    }
}

pub async fn handle(
    axum::extract::State(state): axum::extract::State<
        std::sync::Arc<std::sync::Mutex<crate::subcommand::serve::State>>,
    >,
) -> Result<GetRootOrListPagesResponse, axum::http::StatusCode> {
    let state = state.lock().map_err(|_| axum::http::StatusCode::CONFLICT)?;

    let page_id = crate::page_id::PageId::root();
    match state.page_metas.get(&page_id) {
        Some(page_meta) => {
            let html = crate::page_io::PageIo::read_page_content(&state.config, &page_id)
                .map_err(|_| axum::http::StatusCode::NOT_FOUND)?;

            Ok(GetRootOrListPagesResponse::from(super::get::GetResponse {
                backlinks: state
                    .backlinks
                    .get(&page_id)
                    .map(|set| set.iter().map(|id| id.to_string()).collect::<Vec<String>>())
                    .unwrap_or_default(),
                html,
                id: page_id.to_string(),
                title: page_meta.title.clone().unwrap_or_default(),
            }))
        }
        None => Ok(GetRootOrListPagesResponse::List),
    }
}
