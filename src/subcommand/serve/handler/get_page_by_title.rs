pub struct GetPageByTitleResponse(crate::page_id::PageId);

impl axum::response::IntoResponse for GetPageByTitleResponse {
    fn into_response(self) -> axum::response::Response {
        let mut response = axum::http::StatusCode::FOUND.into_response();
        response.headers_mut().insert(
            axum::http::header::LOCATION,
            // FIXME: unwrap
            axum::http::HeaderValue::from_str(&format!("/pages/{}", self.0)).unwrap(),
        );
        response
    }
}

pub async fn handle(
    axum::extract::State(state): axum::extract::State<
        std::sync::Arc<std::sync::Mutex<crate::subcommand::serve::State>>,
    >,
    axum::extract::Path(title): axum::extract::Path<String>,
) -> Result<GetPageByTitleResponse, axum::http::StatusCode> {
    let state = state.lock().map_err(|_| axum::http::StatusCode::CONFLICT)?;
    let page_ids = state
        .index
        .page_titles
        .get(&title)
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;
    // FIXME: select page_id
    let page_id = page_ids.first().ok_or(axum::http::StatusCode::NOT_FOUND)?;
    Ok(GetPageByTitleResponse(page_id.clone()))
}
