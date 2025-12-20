use anyhow::Context;

#[derive(clap::Args)]
pub(crate) struct Args {
    /// The name to save the image as
    #[arg(long)]
    name: Option<String>,
    /// The path to the image file to add
    path: std::path::PathBuf,
    /// Whether to push the image to remote storage after adding
    #[arg(long)]
    push: bool,
}

pub(super) async fn execute(Args { name, path, push }: Args) -> anyhow::Result<()> {
    let config = crate::config::Config::load().await?;
    let images_dir = config.images_dir();
    tokio::fs::create_dir_all(&images_dir).await?;

    let name = name
        .or_else(|| {
            path.file_name()
                .and_then(|it| it.to_str())
                .map(|it| it.to_owned())
        })
        .context("file_name is none")?;
    let image_file_path = images_dir.join(&name);

    tokio::fs::copy(&path, &image_file_path).await?;

    if push {
        crate::subcommand::image::push::execute(crate::subcommand::image::push::Args {
            name: Some(name),
        })
        .await?;
    }

    Ok(())
}
