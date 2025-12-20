use anyhow::Context;

pub(crate) struct Config {
    data_dir: std::path::PathBuf,
    pub(crate) google_application_credentials: std::path::PathBuf,
    pub(crate) image_bucket_name: String,
    pub(crate) image_object_prefix: String,
    pub(crate) port: Option<u16>,
}

impl Config {
    pub(crate) fn data_dir(&self) -> &std::path::Path {
        &self.data_dir
    }

    pub(crate) fn images_dir(&self) -> std::path::PathBuf {
        self.data_dir.join("images")
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct ConfigJson {
    pub(crate) data_dir: std::path::PathBuf,
    pub(crate) google_application_credentials: std::path::PathBuf,
    pub(crate) image_bucket_name: String,
    pub(crate) image_object_prefix: String,
    pub(crate) port: Option<u16>,
}

impl TryFrom<ConfigJson> for Config {
    type Error = anyhow::Error;

    fn try_from(
        ConfigJson {
            data_dir,
            google_application_credentials,
            image_bucket_name,
            image_object_prefix,
            port,
        }: ConfigJson,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            data_dir,
            google_application_credentials,
            image_bucket_name,
            image_object_prefix,
            port,
        })
    }
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
        let config_json =
            serde_json::from_str::<ConfigJson>(&s).context("failed to parse config file")?;
        let config = Config::try_from(config_json)?;
        Ok(config)
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
    fn test_impl_config_data_dir() -> anyhow::Result<()> {
        let data_dir = <std::path::PathBuf as std::str::FromStr>::from_str("/path/to/data_dir")?;
        let mut config = build_config()?;
        config.data_dir = data_dir.clone();
        assert_eq!(config.data_dir(), data_dir);
        Ok(())
    }

    #[test]
    fn test_impl_config_images_dir() -> anyhow::Result<()> {
        let data_dir = <std::path::PathBuf as std::str::FromStr>::from_str("/path/to/data_dir")?;
        let mut config = build_config()?;
        config.data_dir = data_dir.clone();
        assert_eq!(config.images_dir(), data_dir.join("images"));
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

    fn build_config() -> anyhow::Result<Config> {
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
        Ok(config)
    }
}
