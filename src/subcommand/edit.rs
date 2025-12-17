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
    let editor = shell_words::split(&editor)?;
    std::process::Command::new(&editor[0])
        .args(&editor[1..])
        .arg(path)
        .status()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_shell_words() -> anyhow::Result<()> {
        assert_eq!(shell_words::split("code --wait")?, vec!["code", "--wait"]);
        Ok(())
    }
}
