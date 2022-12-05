use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct AppConfig {
    #[serde(default = "default_app_name")]
    pub app_name: String,
    #[serde(default = "default_app_environment")]
    pub app_environment: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_airtable_api_key")]
    pub airtable_api_key: String,
    #[serde(default = "default_airtable_id")]
    pub airtable_id: String,
}

fn default_app_name() -> String {
    String::from("http-file-server-example")
}

fn default_app_environment() -> String {
    String::from("development")
}

fn default_log_level() -> String {
    String::from("debug")
}

fn default_airtable_api_key() -> String {
    String::from("key")
}

fn default_airtable_id() -> String {
    String::from("app8wLQrrIMrnn673")
}
