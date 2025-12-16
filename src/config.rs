use anyhow::Context;

#[derive(serde::Deserialize)]
pub(crate) struct Config {
    pub(crate) data_dir: std::path::PathBuf,
    pub(crate) google_application_credentials: std::path::PathBuf,
    pub(crate) image_bucket_name: String,
    pub(crate) image_object_prefix: String,
    pub(crate) port: Option<u16>,
}

impl Config {
    pub(crate) async fn load() -> anyhow::Result<Config> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("fubako");
        let config_file_path = xdg_dirs
            .find_config_file("config.json")
            .context("config file not found")?;
        let config_file_content =
            std::fs::read_to_string(config_file_path).context("failed to read config file")?;
        <Self as std::str::FromStr>::from_str(&config_file_content)
    }
}

impl std::str::FromStr for Config {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).context("failed to parse config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_port_is_option() -> anyhow::Result<()> {
        let s = r#"
        {
            "data_dir": "/path/to/data/dir",
            "google_application_credentials": "/path/to/credentials.json",
            "image_bucket_name": "my-image-bucket",
            "image_object_prefix": "images/"
        }
        "#;
        let config = <Config as std::str::FromStr>::from_str(s)?;
        assert_eq!(config.port, None);
        Ok(())
    }

    #[test]
    fn test_impl_config_load() {
        // TODO: Add test for Config::load
    }

    #[test]
    fn test_impl_from_str_for_config() -> anyhow::Result<()> {
        let s = r#"
        {
            "data_dir": "/path/to/data/dir",
            "google_application_credentials": "/path/to/credentials.json",
            "image_bucket_name": "my-image-bucket",
            "image_object_prefix": "images/",
            "port": 8080
        }
        "#;
        let config = <Config as std::str::FromStr>::from_str(s)?;
        assert_eq!(
            config.data_dir,
            std::path::PathBuf::from("/path/to/data/dir")
        );
        assert_eq!(
            config.google_application_credentials,
            std::path::PathBuf::from("/path/to/credentials.json")
        );
        assert_eq!(config.image_bucket_name, "my-image-bucket");
        assert_eq!(config.image_object_prefix, "images/");
        assert_eq!(config.port, Some(8080));
        Ok(())
    }
}
