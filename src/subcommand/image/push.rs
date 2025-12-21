use anyhow::Context as _;

#[derive(clap::Args)]
pub(crate) struct Args {
    /// The name of the image to push
    pub(crate) name: Option<String>,
}

pub(super) async fn execute(args: Args) -> anyhow::Result<()> {
    let config = crate::config::Config::load().await?;
    let image_bucket_name = config.image_sync.bucket_name.clone();
    let image_object_prefix = config.image_sync.object_prefix.clone();
    let images_dir = config.images_dir().canonicalize()?;

    let image_names = {
        let local_image_names = crate::util::list_local_image_names(&images_dir)?;
        let remote_image_names = crate::util::list_remote_image_names(
            &config.image_sync.google_application_credentials,
            &config.image_sync.bucket_name,
            &config.image_sync.object_prefix,
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

    let service_account_key_content =
        std::fs::read_to_string(&config.image_sync.google_application_credentials)
            .context("failed to read service account key file")?;
    let service_account_key =
        serde_json::from_str::<serde_json::Value>(&service_account_key_content)
            .context("failed to parse service account key file")?;
    let client = google_cloud_storage::client::Storage::builder()
        .with_credentials(
            google_cloud_auth::credentials::service_account::Builder::new(service_account_key)
                .build()
                .context("failed to build credentials")?,
        )
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
