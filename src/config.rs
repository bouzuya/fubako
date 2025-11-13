use anyhow::Context;

#[derive(serde::Deserialize)]
pub(crate) struct Config {
    pub(crate) data_dir: std::path::PathBuf,
    pub(crate) image_bucket_name: String,
    pub(crate) image_object_prefix: String,
}

impl Config {
    pub(crate) async fn load() -> anyhow::Result<Config> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("fubako");
        let config_file_path = xdg_dirs
            .find_config_file("config.json")
            .context("config file not found")?;
        let config_file_content =
            std::fs::read_to_string(config_file_path).context("failed to read config file")?;
        Ok(serde_json::from_str(&config_file_content).context("failed to parse config file")?)
    }
}
