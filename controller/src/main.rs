use common_models::{init_logging, load_config, HeartbeatMessage, MqttConfig, ServiceStatus};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use std::collections::HashMap;
use std::sync::{Arc, Mutex}; // Nové: pro bezpečné sdílení paměti
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() {
    init_logging();
    let config: MqttConfig = load_config("controller").unwrap_or_default();

    let mut mqttoptions = MqttOptions::new(&config.client_id, &config.host, config.port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // SDÍLENÁ MAPA: Zabalíme ji do Mutexu a Arc, aby k ní mohl i Watchdog
    let active_services = Arc::new(Mutex::new(HashMap::<String, (HeartbeatMessage, Instant)>::new()));

    // --- WATCHDOG TASK ---
    let watchdog_map = Arc::clone(&active_services);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            let mut map = watchdog_map.lock().unwrap();
            let now = Instant::now();

            for (name, (msg, last_seen)) in map.iter_mut() {
                // Pokud jsme o službě neslyšeli víc než 30 vteřin a není už mrtvá
                if now.duration_since(*last_seen) > Duration::from_secs(30) && msg.status != ServiceStatus::Dead {
                    msg.status = ServiceStatus::Dead;
                    error!("[\u{26B0}] Watchdog: Služba '{}' neodpovídá! Označena za DEAD.", name);
                }
            }
        }
    });

    info!("Controller online. Čekám na heartbeaty...");

    // --- HLAVNÍ MQTT SMYČKA ---
    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                info!("Připojeno! Obnovuji odběry...");
                let c = client.clone();
                tokio::spawn(async move {
                    let _ = c.subscribe("heartbeat/#", QoS::AtMostOnce).await;
                });
            }

            Ok(Event::Incoming(Incoming::Publish(p))) => {
                if p.topic.starts_with("heartbeat/") {
                    if let Ok(heartbeat) = serde_json::from_slice::<HeartbeatMessage>(&p.payload) {
                        let mut map = active_services.lock().unwrap();

                        // ZOTAVENÍ: Pokud byla služba mrtvá a teď přišel heartbeat
                        if let Some((old_msg, _)) = map.get(&heartbeat.service_name) {
                            if old_msg.status == ServiceStatus::Dead {
                                info!("[\u{267B}] Služba '{}' se zotavila (Kubernetes restart?).", heartbeat.service_name);
                            }
                        }

                        // Vyhodnocení stavu (stejné jako minule)
                        match heartbeat.status {
                            ServiceStatus::Ok => info!("[\u{2705}] {} ok", heartbeat.service_name),
                            ServiceStatus::Degraded => warn!("[\u{26A0}] {} degraded", heartbeat.service_name),
                            ServiceStatus::Error => error!("[\u{1F6A8}] {} error", heartbeat.service_name),
                            _ => {}
                        }

                        // Aktualizace mapy (zpráva + aktuální čas "last seen")
                        map.insert(heartbeat.service_name.clone(), (heartbeat, Instant::now()));
                    }
                }
            }
            Err(e) => {
                error!("Chyba sítě: {:?}. Reconnect...", e);
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
            _ => {}
        }
    }
}
