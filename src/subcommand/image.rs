use std::io::Write;

use anyhow::Context;

#[derive(clap::Subcommand)]
pub(crate) enum ImageCommand {
    /// Download all images from the bucket
    Download,
    /// Upload an image
    Upload { file_name: std::path::PathBuf },
}

pub(super) async fn execute(command: ImageCommand) -> anyhow::Result<()> {
    match command {
        ImageCommand::Download => {
            let config = crate::config::Config::load().await?;
            let image_bucket_name = config.image_bucket_name.clone();
            let image_object_prefix = config.image_object_prefix.clone();
            let images_dir = config.data_dir.join("images");
            tokio::fs::create_dir_all(&images_dir).await?;

            let client = google_cloud_storage::client::Storage::builder()
                .build()
                .await?;

            use google_cloud_gax::paginator::ItemPaginator;
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

                let mut read_object_response = client
                    .read_object(
                        format!("projects/_/buckets/{image_bucket_name}"),
                        object.name.clone(),
                    )
                    .send()
                    .await?;
                // TODO: check file_name
                let file_name = object.name.split("/").last().context("file name")?;
                let image_file_path = images_dir.join(file_name);
                let mut file = std::fs::File::create(&image_file_path)?;
                while let Some(bytes) = read_object_response.next().await.transpose()? {
                    file.write(&bytes)?;
                }
                println!("Downloaded image to: {}", image_file_path.display());
            }
        }
        ImageCommand::Upload { file_name } => {
            let config = crate::config::Config::load().await?;
            let image_bucket_name = config.image_bucket_name.clone();
            let image_object_prefix = config.image_object_prefix.clone();
            let images_dir = config.data_dir.join("images").canonicalize()?;
            let image_path = images_dir.join(file_name).canonicalize()?;
            if !image_path.starts_with(images_dir) {
                return Err(anyhow::anyhow!("Invalid image path"));
            }

            let client = google_cloud_storage::client::Storage::builder()
                .build()
                .await?;
            let object_name = image_path
                .file_name()
                .context("file_name")?
                .to_str()
                .context("file_name is not UTF-8")?;
            let payload = tokio::fs::File::open(&image_path).await?;
            let object = client
                .write_object(
                    format!("projects/_/buckets/{image_bucket_name}"),
                    format!("{image_object_prefix}{object_name}"),
                    payload,
                )
                .send_buffered()
                .await?;
            println!("Uploaded image to: {}/{}", object.bucket, object.name);
        }
    }
    Ok(())
}
