use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize};
use uuid::Uuid;
/// Sdílená konfigurace pro připojení k MQTT brokeru.
/// #[serde(default)] zaručí, že pokud něco v konfiguraci chybí, použije se trait Default.
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,

    pub ca_cert_path: Option<String>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
    pub insecure_skip_verify: Option<bool>,
    pub topic: Option<String>,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 1883,
            client_id: format!("dashboard-{}", Uuid::new_v4().simple()),
            username: None,
            password: None,
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
            insecure_skip_verify: None,
            topic: None,
        }
    }
}

/// Načte konfiguraci v kaskádě: default soubor -> app soubor -> ENV proměnné.
pub fn load_config<'a, T: serde::Deserialize<'a>>(app_name: &str) -> Result<T, ConfigError> {
    let builder = Config::builder()
        .add_source(File::with_name("config/default").required(false))
        .add_source(File::with_name(&format!("config/{}", app_name)).required(false))
        .add_source(
            Environment::with_prefix("DASHBOARD")
                .separator("__") // Podpora pro vnořené struktury (jako HashMap) přes ENV
                .ignore_empty(true),
        );

    let config = builder.build()?;
    config.try_deserialize::<T>()
}
