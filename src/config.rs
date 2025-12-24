use anyhow::Context;

#[derive(Clone)]
pub(crate) struct Config {
    data_dir: std::path::PathBuf,
    image_sync: Option<ConfigImageSync>,
    port: Option<u16>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ConfigImageSync {
    pub(crate) bucket_name: String,
    pub(crate) google_application_credentials: std::path::PathBuf,
    pub(crate) object_prefix: String,
}

impl Config {
    pub(crate) async fn load_from(path: &std::path::Path) -> anyhow::Result<Self> {
        let config_file_content = tokio::fs::read_to_string(path)
            .await
            .context("failed to read config file")?;
        <Self as std::str::FromStr>::from_str(&config_file_content)
    }

    pub(crate) async fn load() -> anyhow::Result<Self> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("fubako");
        let config_file_path = xdg_dirs
            .find_config_file("config.json")
            .context("config file not found")?;
        Self::load_from(&config_file_path).await
    }

    pub(crate) fn data_dir(&self) -> &std::path::Path {
        &self.data_dir
    }

    pub(crate) fn image_sync(&self) -> Option<ConfigImageSync> {
        self.image_sync.clone()
    }

    pub(crate) fn images_dir(&self) -> std::path::PathBuf {
        self.data_dir.join("images")
    }

    pub(crate) fn port(&self) -> u16 {
        self.port.unwrap_or(3000_u16)
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

#[derive(serde::Deserialize)]
struct ConfigJson {
    data_dir: std::path::PathBuf,
    image_sync: Option<ConfigImageSyncJson>,
    port: Option<u16>,
}

#[derive(Debug, PartialEq, serde::Deserialize)]
struct ConfigImageSyncJson {
    bucket_name: String,
    google_application_credentials: std::path::PathBuf,
    object_prefix: String,
}

impl TryFrom<ConfigJson> for Config {
    type Error = anyhow::Error;

    fn try_from(
        ConfigJson {
            data_dir,
            image_sync,
            port,
        }: ConfigJson,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            data_dir,
            image_sync: image_sync.map(
                |ConfigImageSyncJson {
                     bucket_name,
                     google_application_credentials,
                     object_prefix,
                 }| {
                    ConfigImageSync {
                        bucket_name,
                        google_application_credentials,
                        object_prefix,
                    }
                },
            ),
            port,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_data_dir() -> anyhow::Result<()> {
        let s = r#"
        {
            "data_dir": "/path/to/data/dir"
        }
        "#;
        let config = <Config as std::str::FromStr>::from_str(s)?;
        assert_eq!(
            config.data_dir(),
            std::path::PathBuf::from("/path/to/data/dir").as_path()
        );
        Ok(())
    }

    #[test]
    fn test_config_image_sync() -> anyhow::Result<()> {
        let s = r#"
        {
            "data_dir": "/path/to/data/dir"
        }
        "#;
        let config = <Config as std::str::FromStr>::from_str(s)?;
        assert!(config.image_sync().is_none());

        let s = r#"
        {
            "data_dir": "/path/to/data/dir",
            "image_sync": {
                "bucket_name": "my-image-bucket",
                "google_application_credentials": "/path/to/credentials.json",
                "object_prefix": "images/"
            }
        }
        "#;
        let config = <Config as std::str::FromStr>::from_str(s)?;
        assert_eq!(
            config.image_sync(),
            Some(ConfigImageSync {
                bucket_name: "my-image-bucket".to_owned(),
                google_application_credentials: std::path::PathBuf::from(
                    "/path/to/credentials.json"
                ),
                object_prefix: "images/".to_owned(),
            })
        );
        Ok(())
    }

    #[test]
    fn test_config_images_dir() -> anyhow::Result<()> {
        let s = r#"
        {
            "data_dir": "/path/to/data/dir"
        }
        "#;
        let config = <Config as std::str::FromStr>::from_str(s)?;
        assert_eq!(
            config.images_dir(),
            std::path::PathBuf::from("/path/to/data/dir/images")
        );
        Ok(())
    }

    #[test]
    fn test_config_port() -> anyhow::Result<()> {
        let s = r#"
        {
            "data_dir": "/path/to/data/dir"
        }
        "#;
        let config = <Config as std::str::FromStr>::from_str(s)?;
        assert_eq!(config.port(), 3000);

        let s = r#"
        {
            "data_dir": "/path/to/data/dir",
            "port": 8080
        }
        "#;
        let config = <Config as std::str::FromStr>::from_str(s)?;
        assert_eq!(config.port(), 8080);
        Ok(())
    }

    #[test]
    fn test_impl_config_load() {
        // TODO: Add test for Config::load
    }

    #[tokio::test]
    async fn test_impl_config_load_from() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let config_path = temp_dir.path().join("config.json");

        let config_content = r#"
        {
            "data_dir": "/path/to/data/dir"
        }
        "#;
        tokio::fs::write(&config_path, config_content).await?;

        let config = Config::load_from(&config_path).await?;
        assert_eq!(
            config.data_dir(),
            std::path::PathBuf::from("/path/to/data/dir").as_path()
        );

        Ok(())
    }
}
