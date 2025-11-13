use anyhow::Context as _;

pub(crate) fn list_local_image_names(
    images_dir: &std::path::Path,
) -> anyhow::Result<std::collections::BTreeSet<String>> {
    let mut image_names = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(images_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let file_name = entry.file_name();
        let file_name = file_name.to_str().context("file_name is not UTF-8")?;
        image_names.insert(file_name.to_owned());
    }
    Ok(image_names)
}

pub(crate) async fn list_remote_image_names(
    google_application_credentials: &std::path::Path,
    image_bucket_name: &str,
    image_object_prefix: &str,
) -> anyhow::Result<std::collections::BTreeSet<String>> {
    use google_cloud_gax::paginator::ItemPaginator;

    let service_account_key_content = std::fs::read_to_string(google_application_credentials)
        .context("failed to read service account key file")?;
    let service_account_key =
        serde_json::from_str::<serde_json::Value>(&service_account_key_content)
            .context("failed to parse service account key file")?;

    let mut image_names = std::collections::BTreeSet::new();
    let control = google_cloud_storage::client::StorageControl::builder()
        .with_credentials(
            google_cloud_auth::credentials::service_account::Builder::new(service_account_key)
                .build()
                .context("failed to build credentials")?,
        )
        .build()
        .await?;
    let builder = control
        .list_objects()
        .set_parent(format!("projects/_/buckets/{image_bucket_name}"))
        .set_prefix(image_object_prefix);
    let mut items = builder.by_item();
    while let Some(result) = items.next().await {
        let object = result?;
        let image_name = object
            .name
            .strip_prefix(image_object_prefix)
            .context("object.name does not start with image_object_prefix")?
            .to_owned();
        if image_name.contains('/') {
            continue;
        }
        image_names.insert(image_name);
    }

    Ok(image_names)
}
