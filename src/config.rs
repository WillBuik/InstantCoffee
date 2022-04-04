use std::{fmt::Debug, {path::PathBuf, ops::Deref}, ops::DerefMut};

use serde::{Serialize, de::DeserializeOwned};
use tokio::io::Result;

/// Trait for file backed configuration files.
pub trait ConfigFile {
    fn path() -> PathBuf;

    const PRETTY: bool;
}

/// InstantCoffee configuration.
pub struct Config<T> where
    T: Serialize + DeserializeOwned
{
    configuration: T,
}

impl <T: Serialize + DeserializeOwned + PartialEq> PartialEq for Config<T> {
    fn eq(&self, other: &Self) -> bool {
        self.configuration == other.configuration
    }
}

impl <T: Serialize + DeserializeOwned + Debug> Debug for Config<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.configuration.fmt(f)
    }
}

impl <T: Serialize + DeserializeOwned + Default> Default for Config<T> {
    fn default() -> Self {
        Self { configuration: Default::default() }
    }
}

impl <T: Serialize + DeserializeOwned> Deref for Config<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        return &self.configuration;
    }
}

impl <T: Serialize + DeserializeOwned> DerefMut for Config<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.configuration;
    }
}

impl <T: Serialize + DeserializeOwned> Config<T> {
    /// Save configuration as JSON.
    pub fn to_json (&self) -> Result<String> {
        return Ok(serde_json::to_string(&self.configuration)?);
    }

    /// Save configuration as JSON.
    pub fn to_json_pretty (&self) -> Result<String> {
        return Ok(serde_json::to_string_pretty(&self.configuration)?);
    }

    /// Load configuration from a JSON str.
    pub fn from_json_str(json: &str) -> Result<Config<T>> {
        return Ok(Config {
            configuration: serde_json::from_str::<T>(&json)?,
        });
    }

    /// Load configuration from a JSON slice.
    pub fn from_json_slice(json_buffer: &[u8]) -> Result<Config<T>> {
        return Ok(Config {
            configuration: serde_json::from_slice::<T>(&json_buffer)?,
        });
    }
}

impl <T: Serialize + DeserializeOwned + ConfigFile> Config<T> {
    /// Save configuration as JSON.
    pub async fn save_file (&self) -> Result<()> {
        let path = T::path();
        if T::PRETTY {
            return tokio::fs::write(path, serde_json::to_string_pretty(&self.configuration)?).await;
        } else {
            return tokio::fs::write(path, serde_json::to_string(&self.configuration)?).await;
        }
    }

    /// Load configuration from JSON.
    pub async fn load_file() -> Result<Config<T>> {
        let path = T::path();
        return Ok(Config {
            configuration: serde_json::from_str::<T>(&tokio::fs::read_to_string(path).await?)?
        });
    }
}

#[cfg(test)]
mod tests {
    use serde::{Serialize, Deserialize};

    use crate::test_utils::CleanupFile;

    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestConfig {
        hello: String,
        world: String,
    }

    impl Default for TestConfig {
        fn default() -> Self {
            Self { hello: "hello".to_string(), world: "world".to_string() }
        }
    }

    impl ConfigFile for TestConfig {
        const PRETTY: bool = true;

        fn path() -> PathBuf {
            return PathBuf::from("testconfig.json");
        }
    }

    #[test]
    fn config() -> Result<()> {
        let config_file = Config::<TestConfig>::default();

        let config_json_compact = "{\"hello\":\"hello\",\"world\":\"world\"}";
        let config_json_pretty = "{\n  \"hello\": \"hello\",\n  \"world\": \"world\"\n}";

        assert_eq!(
            config_file.to_json()?,
            config_json_compact.to_string(),
            "Config.to_json()");

        assert_eq!(
            config_file.to_json_pretty()?,
            config_json_pretty.to_string(),
            "Config.to_json_pretty()");
        
        let config_file_from_json = Config::<TestConfig>::from_json_str(config_json_compact)?;
        assert_eq!(config_file, config_file_from_json, "Config::from_json_str()");

        let config_file_from_json = Config::<TestConfig>::from_json_slice(&config_json_compact.as_bytes())?;
        assert_eq!(config_file, config_file_from_json, "Config::from_json_slice()");

        Ok(())
    }

    #[tokio::test]
    async fn config_file() -> Result<()> {
        let _cleanup = CleanupFile::from(TestConfig::path().as_path());

        let mut config_file = Config::<TestConfig>::default();
        config_file.hello = "goodbye".to_string();

        config_file.save_file().await?;
        let config_file_loaded = Config::<TestConfig>::load_file().await?;
        assert_eq!(config_file, config_file_loaded, "ConfigFile load/save");

        Ok(())
    }
}
