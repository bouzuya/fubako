#[derive(clap::Args)]
pub(crate) struct Args {
    /// The name of the image to push
    name: Option<String>,
}

pub(super) async fn execute(args: Args) -> anyhow::Result<()> {
    let config = crate::config::Config::load().await?;
    let image_bucket_name = config.image_bucket_name.clone();
    let image_object_prefix = config.image_object_prefix.clone();
    let images_dir = config.data_dir.join("images").canonicalize()?;

    let image_names = {
        let local_image_names = crate::util::list_local_image_names(&images_dir)?;
        let remote_image_names = crate::util::list_remote_image_names(
            &config.image_bucket_name,
            &config.image_object_prefix,
        )
        .await?;
        local_image_names
            .into_iter()
            .filter(|it| {
                args.name.as_ref().map(|name| it == name).unwrap_or(true)
                    && !remote_image_names.contains(it)
            })
            .collect::<std::collections::BTreeSet<String>>()
    };

    let client = google_cloud_storage::client::Storage::builder()
        .build()
        .await?;
    for image_name in image_names {
        let image_path = images_dir.join(&image_name).canonicalize()?;
        let payload = tokio::fs::File::open(&image_path).await?;
        let object = client
            .write_object(
                format!("projects/_/buckets/{image_bucket_name}"),
                format!("{image_object_prefix}{image_name}"),
                payload,
            )
            .send_buffered()
            .await?;
        println!("Uploaded image to: {}/{}", object.bucket, object.name);
    }
    Ok(())
}
