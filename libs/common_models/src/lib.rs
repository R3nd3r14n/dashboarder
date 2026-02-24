// Deklarujeme moduly (soubory), které kompilátor musí zpracovat
pub mod config;
pub mod messages;
pub mod system;

// (Volitelné) Tzv. "Re-export".
// Díky tomuto si programátor v jiné mikroslužbě nemusí pamatovat,
// ve kterém souboru to přesně je. Může napsat jen:
// use common_models::{DashboardMessage, MqttConfig, HeartbeatMessage};
pub use config::{load_config, MqttConfig};
pub use messages::{DashboardMessage, MessageType};
pub use system::{HeartbeatMessage, LogLevel, LogMessage, ServiceStatus};
