use std::io::Write;

use anyhow::Context;

#[derive(clap::Args)]
pub(crate) struct Args {
    /// The name of the image to pull
    name: Option<String>,
}

pub(super) async fn execute(args: Args) -> anyhow::Result<()> {
    let config = crate::config::Config::load().await?;
    let image_sync_config = match config.image_sync() {
        Some(it) => it,
        None => {
            anyhow::bail!("image sync is not configured");
        }
    };
    let image_bucket_name = image_sync_config.bucket_name.clone();
    let images_dir = config.images_dir();
    tokio::fs::create_dir_all(&images_dir).await?;

    let image_names = {
        let remote_image_names = crate::util::list_remote_image_names(
            &image_sync_config.google_application_credentials,
            &image_sync_config.bucket_name,
            &image_sync_config.object_prefix,
        )
        .await?;
        let local_image_names = crate::util::list_local_image_names(&images_dir)?;
        remote_image_names
            .into_iter()
            .filter(|it| {
                args.name.as_ref().map(|name| it == name).unwrap_or(true)
                    && !local_image_names.contains(it)
            })
            .collect::<std::collections::BTreeSet<String>>()
    };

    let service_account_key_content =
        std::fs::read_to_string(&image_sync_config.google_application_credentials)
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
        let mut read_object_response = client
            .read_object(
                format!("projects/_/buckets/{image_bucket_name}"),
                format!("{}{}", image_sync_config.object_prefix, image_name),
            )
            .send()
            .await?;
        // TODO: check file_name
        let file_name = image_name.split("/").last().context("file name")?;
        let image_file_path = images_dir.join(file_name);
        let mut file = std::fs::File::create(&image_file_path)?;
        while let Some(bytes) = read_object_response.next().await.transpose()? {
            file.write(&bytes)?;
        }
        println!("Downloaded image to: {}", image_file_path.display());
    }
    Ok(())
}
