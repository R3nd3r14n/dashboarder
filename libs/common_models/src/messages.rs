use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

// ==========================================
// 1. DATOVÝ KONTRAKT (ZPRÁVY PRO MQTT)
// ==========================================

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Data,
    Event,
    Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardMessage {
    id: Uuid,
    create_timestamp: DateTime<Utc>,
    msg_type: MessageType,

    #[serde(skip_serializing_if = "Option::is_none")]
    mod_timestamp: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    last_access_timestamp: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    storage: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    other: Option<Value>,
}

impl DashboardMessage {
    pub fn new(msg_type: MessageType) -> Self {
        Self {
            id: Uuid::new_v4(),
            create_timestamp: Utc::now(),
            msg_type,
            mod_timestamp: None,
            last_access_timestamp: None,
            value: None,
            source: None,
            storage: None,
            metadata: None,
            other: None,
        }
    }

    fn touch_mod(&mut self) {
        self.mod_timestamp = Some(Utc::now());
    }

    pub fn set_value(&mut self, value: Value) {
        self.value = Some(value);
        self.touch_mod();
    }

    pub fn set_source(&mut self, source: &str) {
        self.source = Some(source.to_string());
        self.touch_mod();
    }

    pub fn set_storage(&mut self, storage: &str) {
        self.storage = Some(storage.to_string());
        self.touch_mod();
    }

    pub fn add_metadata(&mut self, key: &str, value: Value) {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
        }
        if let Some(map) = &mut self.metadata {
            map.insert(key.to_string(), value);
            self.touch_mod();
        }
    }

    pub fn set_other(&mut self, other_data: Value) {
        self.other = Some(other_data);
        self.touch_mod();
    }

    pub fn get_value(&mut self) -> Option<&Value> {
        self.last_access_timestamp = Some(Utc::now());
        self.value.as_ref()
    }

    pub fn get_other(&mut self) -> Option<&Value> {
        self.last_access_timestamp = Some(Utc::now());
        self.other.as_ref()
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn create_timestamp(&self) -> DateTime<Utc> {
        self.create_timestamp
    }
}
