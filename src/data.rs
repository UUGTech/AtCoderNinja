use crate::{
    ac_scraper::get_local_session,
    config::{get_config, ConfigMap, ConfigStrMap, ToHashMapString},
};
use anyhow::Result;
use reqwest::{header::HeaderMap, Client};

pub struct ACS {
    pub config_map: ConfigMap,
    pub config_str_map: ConfigStrMap,
    pub client: Client,
    pub cookies: Option<HeaderMap>,
}

const USER_AGENT: &str = "ac-ninja";

impl ACS {
    pub async fn new() -> Result<Self> {
        let config_map: ConfigMap = get_config().unwrap();
        let config_str_map = config_map.to_hash_map_string();
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .cookie_store(true)
            .build()
            .unwrap();
        let cookies = get_local_session()?;
        if cookies.is_some() {
            return Ok(ACS {
                config_map,
                config_str_map,
                client,
                cookies,
            });
        } else {
            return Ok(ACS {
                config_map,
                config_str_map,
                client,
                cookies: None,
            });
        }
    }
}
