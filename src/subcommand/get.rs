#[derive(clap::Args)]
pub(crate) struct Args {
    /// The page ID to show
    page_id: crate::page_id::PageId,
}

pub(super) async fn execute(Args { page_id }: Args) -> anyhow::Result<()> {
    let config = crate::config::Config::load().await?;
    let content = crate::page_io::PageIo::read_page_raw_content(&config, &page_id)?;
    println!("{}", content);
    Ok(())
}
