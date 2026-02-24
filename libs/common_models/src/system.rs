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
