pub async fn handle(
    axum::extract::State(state): axum::extract::State<
        std::sync::Arc<std::sync::Mutex<crate::subcommand::serve::State>>,
    >,
    axum::extract::Path(image_name): axum::extract::Path<String>,
) -> Result<Vec<u8>, axum::http::StatusCode> {
    let state = state.lock().map_err(|_| axum::http::StatusCode::CONFLICT)?;
    let images_dir = state
        .config
        .images_dir()
        .canonicalize()
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    let image_file_path = images_dir
        .join(&image_name)
        .canonicalize()
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    if !image_file_path.starts_with(images_dir) {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }

    if !std::fs::exists(&image_file_path)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(axum::http::StatusCode::NOT_FOUND);
    }

    Ok(std::fs::read(&image_file_path)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?)
}
