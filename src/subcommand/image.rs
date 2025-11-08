mod pull;
mod push;

#[derive(clap::Subcommand)]
pub(crate) enum Subcommand {
    /// Pull images from remote storage
    Pull(self::pull::Args),
    /// Push images to remote storage
    Push(self::push::Args),
}

pub(super) async fn execute(subcommand: Subcommand) -> anyhow::Result<()> {
    match subcommand {
        Subcommand::Pull(args) => self::pull::execute(args).await,
        Subcommand::Push(args) => self::push::execute(args).await,
    }
}
