pub enum GetRootOrListPagesResponse {
    Get(super::get::GetResponse),
    List(super::list::ListResponse),
}

impl From<super::get::GetResponse> for GetRootOrListPagesResponse {
    fn from(value: super::get::GetResponse) -> Self {
        Self::Get(value)
    }
}

impl From<super::list::ListResponse> for GetRootOrListPagesResponse {
    fn from(value: super::list::ListResponse) -> Self {
        Self::List(value)
    }
}

impl axum::response::IntoResponse for GetRootOrListPagesResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Get(resp) => resp.into_response(),
            Self::List(resp) => resp.into_response(),
        }
    }
}

pub async fn handle(
    axum::extract::State(state): axum::extract::State<
        std::sync::Arc<std::sync::Mutex<crate::subcommand::serve::State>>,
    >,
    axum::extract::Query(super::list::ListRequestQuery { q }): axum::extract::Query<
        super::list::ListRequestQuery,
    >,
) -> Result<GetRootOrListPagesResponse, axum::http::StatusCode> {
    let q = q.unwrap_or_default().trim().to_owned();
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
        None => {
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
                .map(|(id, meta)| super::list::ListResponsePageMeta {
                    id: id.to_string(),
                    title: meta.title.clone().unwrap_or_default(),
                })
                .collect::<Vec<super::list::ListResponsePageMeta>>();
            Ok(GetRootOrListPagesResponse::from(
                super::list::ListResponse { page_metas, q },
            ))
        }
    }
}
