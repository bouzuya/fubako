pub async fn handle() -> axum::response::Response<axum::body::Body> {
    let mut response = axum::response::IntoResponse::into_response(include_str!(
        "../../../../public/styles/index.css"
    ));
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("text/css"),
    );
    response
}
