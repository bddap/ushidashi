use serde::{Deserialize, Serialize};
use xdg::BaseDirectories;

#[derive(Serialize, Deserialize, Debug)]
pub struct Secrets {
    pub openai_api_key: String,
}

impl Secrets {
    pub fn load() -> anyhow::Result<Self> {
        let base = BaseDirectories::with_prefix("techakiga")?;
        let path = base
            .find_config_file("secrets.toml")
            .ok_or(anyhow::anyhow!(
                "No secrets.toml file found. Try logging in with 'refac login'.",
            ))?;
        let secrets = std::fs::read_to_string(path)?;
        let ret: Secrets = toml::from_str(&secrets)?;
        Ok(ret)
    }
}
