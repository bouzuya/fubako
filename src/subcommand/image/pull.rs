use std::io::Write;

use anyhow::Context;

pub(super) async fn execute() -> anyhow::Result<()> {
    let config = crate::config::Config::load().await?;
    let image_bucket_name = config.image_bucket_name.clone();
    let images_dir = config.data_dir.join("images");
    tokio::fs::create_dir_all(&images_dir).await?;

    let image_names = {
        let remote_image_names = crate::util::list_remote_image_names(
            &config.image_bucket_name,
            &config.image_object_prefix,
        )
        .await?;
        let local_image_names = crate::util::list_local_image_names(&images_dir)?;
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
    Ok(())
}
