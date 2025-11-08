mod image;
mod new;
mod preview;

#[derive(clap::Subcommand)]
pub(crate) enum Subcommand {
    /// Manage images
    #[clap(subcommand)]
    Image(self::image::Subcommand),
    /// Create a new page
    New,
    /// Start a local preview server
    Preview,
}

impl Subcommand {
    pub(crate) async fn execute(self) -> anyhow::Result<()> {
        match self {
            Subcommand::Image(subcommand) => self::image::execute(subcommand).await,
            Subcommand::New => self::new::execute().await,
            Subcommand::Preview => self::preview::execute().await,
        }
    }
}
