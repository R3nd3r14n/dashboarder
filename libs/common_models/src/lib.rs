// Deklarujeme moduly (soubory), které kompilátor musí zpracovat
pub mod config;
pub mod messages;
pub mod mqtt;
pub mod system;

// (Volitelné) Tzv. "Re-export".
// Díky tomuto si programátor v jiné mikroslužbě nemusí pamatovat,
// ve kterém souboru to přesně je. Může napsat jen:
// use common_models::{DashboardMessage, MqttConfig, HeartbeatMessage};
pub use config::{load_config, MqttConfig};
pub use messages::{DashboardMessage, MessageType};
pub use system::{init_logging, HeartbeatMessage, LogLevel, LogMessage, ServiceStatus};
// Nové re-exporty pro síťařinu
pub use mqtt::connect_mqtt;

// FINTA: Vyexportujeme samotnou knihovnu paho_mqtt.
// Díky tomu si parser nemusí paho_mqtt stahovat sám,
// ale použije ho přímo jako common_models::paho_mqtt.
pub use paho_mqtt;
