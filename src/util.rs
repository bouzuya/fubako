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
    image_bucket_name: &str,
    image_object_prefix: &str,
) -> anyhow::Result<std::collections::BTreeSet<String>> {
    use google_cloud_gax::paginator::ItemPaginator;

    let mut image_names = std::collections::BTreeSet::new();
    let control = google_cloud_storage::client::StorageControl::builder()
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
