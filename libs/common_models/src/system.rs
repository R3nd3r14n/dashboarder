use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ==========================================
// STAVY SLUŽEB (Pro Heartbeat)
// ==========================================

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Ok,
    Degraded, // Např. jede, ale spadlo připojení k databázi
    Error,    // Fatální problém
    Dead,     // Tohle si může Controller nastavit interně, když služba přestane posílat heartbeat
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeartbeatMessage {
    pub service_name: String,
    pub status: ServiceStatus,
    pub uptime_seconds: u64,
    pub timestamp: DateTime<Utc>,
    /// Volitelné pole pro stručný popis (např. "Ztratil jsem spojení s API")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
}

impl HeartbeatMessage {
    pub fn new(service_name: &str, status: ServiceStatus, uptime_seconds: u64) -> Self {
        Self {
            service_name: service_name.to_string(),
            status,
            uptime_seconds,
            timestamp: Utc::now(),
            status_message: None,
        }
    }
}

// ==========================================
// UNIVERZÁLNÍ LOGY (Pro Syslog Storager)
// ==========================================

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogMessage {
    pub service_name: String,
    pub level: LogLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    /// Třeba název souboru a řádek, kde chyba vznikla
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_path: Option<String>,
}

impl LogMessage {
    pub fn new(service_name: &str, level: LogLevel, message: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            level,
            message: message.to_string(),
            timestamp: Utc::now(),
            module_path: None,
        }
    }
}

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_logging() {
    // Toto nastavení umožní ovládat detailnost logů přes ENV proměnnou RUST_LOG
    // Např. RUST_LOG=debug cargo run ...
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer()) // Logování do konzole
        // Sem v budoucnu přidáme .with(MqttLayer::new())
        .init();

    tracing::info!("Logování inicializováno.");
}
