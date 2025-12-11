mod edit;
mod image;
mod new;
mod serve;

#[derive(clap::Subcommand)]
pub(crate) enum Subcommand {
    /// Edit the page
    Edit(self::edit::Args),
    /// Manage images
    #[command(subcommand)]
    Image(self::image::Subcommand),
    /// Create a new page
    New,
    /// Start the local server
    Serve,
}

impl Subcommand {
    pub(crate) async fn execute(self) -> anyhow::Result<()> {
        match self {
            Subcommand::Edit(args) => self::edit::execute(args).await,
            Subcommand::Image(subcommand) => self::image::execute(subcommand).await,
            Subcommand::New => self::new::execute().await,
            Subcommand::Serve => self::serve::execute().await,
        }
    }
}
