use std::io::Write;

use anyhow::Context;

#[derive(clap::Subcommand)]
pub(crate) enum ImageCommand {
    /// Pull images from remote storage
    Pull,
    /// Push images to remote storage
    Push,
}

pub(super) async fn execute(command: ImageCommand) -> anyhow::Result<()> {
    match command {
        ImageCommand::Pull => {
            let config = crate::config::Config::load().await?;
            let image_bucket_name = config.image_bucket_name.clone();
            let images_dir = config.data_dir.join("images");
            tokio::fs::create_dir_all(&images_dir).await?;

            let image_names = {
                let remote_image_names =
                    list_remote_image_names(&config.image_bucket_name, &config.image_object_prefix)
                        .await?;
                let local_image_names = list_local_image_names(&images_dir)?;
                remote_image_names
                    .into_iter()
                    .filter(|it| !local_image_names.contains(it))
                    .collect::<std::collections::BTreeSet<String>>()
            };

            let client = google_cloud_storage::client::Storage::builder()
                .build()
                .await?;
            for image_name in image_names {
                let mut read_object_response = client
                    .read_object(
                        format!("projects/_/buckets/{image_bucket_name}"),
                        format!("{}{}", config.image_object_prefix, image_name),
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
        }
        ImageCommand::Push => {
            let config = crate::config::Config::load().await?;
            let image_bucket_name = config.image_bucket_name.clone();
            let image_object_prefix = config.image_object_prefix.clone();
            let images_dir = config.data_dir.join("images").canonicalize()?;

            let image_names = {
                let local_image_names = list_local_image_names(&images_dir)?;
                let remote_image_names =
                    list_remote_image_names(&config.image_bucket_name, &config.image_object_prefix)
                        .await?;
                local_image_names
                    .into_iter()
                    .filter(|it| !remote_image_names.contains(it))
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
        }
    }
    Ok(())
}

fn list_local_image_names(
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

async fn list_remote_image_names(
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
