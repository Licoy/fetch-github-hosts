use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_lang")]
    pub lang: String,
    #[serde(default)]
    pub client: ClientConfig,
    #[serde(default)]
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    #[serde(default = "default_interval")]
    pub interval: u32,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default = "default_select_origin")]
    pub select_origin: String,
    #[serde(default)]
    pub custom_url: String,
    #[serde(default)]
    pub auto_fetch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_interval")]
    pub interval: u32,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub template_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            lang: default_lang(),
            client: ClientConfig::default(),
            server: ServerConfig::default(),
        }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            interval: default_interval(),
            method: default_method(),
            select_origin: default_select_origin(),
            custom_url: String::new(),
            auto_fetch: false,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            interval: default_interval(),
            port: default_port(),
            template_path: String::new(),
        }
    }
}

fn default_lang() -> String {
    "zh-CN".to_string()
}

fn default_interval() -> u32 {
    60
}

fn default_method() -> String {
    "official".to_string()
}

fn default_select_origin() -> String {
    "FetchGithubHosts".to_string()
}

fn default_port() -> u16 {
    9898
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub has_update: bool,
    pub version: String,
    pub url: String,
}

/// Log payload with i18n support and log level
#[derive(Debug, Clone, Serialize)]
pub struct LogPayload {
    /// i18n key, e.g., "client.fetchSuccess"
    pub key: String,
    /// Interpolation parameters for the i18n key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    /// Log level: "info", "success", "error"
    pub level: String,
}
