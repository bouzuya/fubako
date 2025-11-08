mod pull;
mod push;

#[derive(clap::Subcommand)]
pub(crate) enum Subcommand {
    /// Pull images from remote storage
    Pull,
    /// Push images to remote storage
    Push,
}

pub(super) async fn execute(subcommand: Subcommand) -> anyhow::Result<()> {
    match subcommand {
        Subcommand::Pull => self::pull::execute().await,
        Subcommand::Push => self::push::execute().await,
    }
}
