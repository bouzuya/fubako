#[derive(clap::Args)]
pub(crate) struct Args {
    /// The editor to use
    #[clap(env, long)]
    editor: String,
    /// The page ID to edit
    page_id: crate::page_id::PageId,
}

pub(super) async fn execute(Args { editor, page_id }: Args) -> anyhow::Result<()> {
    let config = crate::config::Config::load().await?;
    let path = crate::page_io::PageIo::page_path(&config, &page_id);
    std::process::Command::new(editor).arg(path).status()?;
    Ok(())
}
