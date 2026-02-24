use common_models::{load_config, HeartbeatMessage, MqttConfig};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use std::collections::HashMap;
use std::time::Duration;

// Tohle makro říká Rustu: "Tato funkce je asynchronní a poběží uvnitř Tokio runtime"
#[tokio::main]
async fn main() {
    // 1. NAČTENÍ KONFIGURACE
    // Zkusíme načíst z config/controller.toml nebo ENV.
    // Pokud nic není, použijeme náš Default (127.0.0.1:1883).
    let config: MqttConfig = load_config("controller").unwrap_or_default();

    println!("Startuji Controller...");
    println!("Připojuji se na MQTT broker: {}:{}", config.host, config.port);

    // 2. NASTAVENÍ MQTT PŘIPOJENÍ
    let mut mqttoptions = MqttOptions::new(&config.client_id, &config.host, config.port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    // AsyncClient::new vrátí klienta (pro odesílání) a eventloop (pro přijímání)
    // To číslo 10 je velikost fronty v paměti pro zprávy.
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // 3. PŘIHLÁŠENÍ K ODBĚRU (Subscribe)
    // Přihlásíme se k odběru všech topiců, které začínají na "heartbeat/"
    client
        .subscribe("heartbeat/#", QoS::AtMostOnce)
        .await
        .expect("Nepodařilo se přihlásit k odběru MQTT!");

    // 4. PAMĚŤ CONTROLLERA (Stavový automat)
    // Zde budeme držet mapu všech služeb a jejich posledního známého stavu
    let mut active_services: HashMap<String, HeartbeatMessage> = HashMap::new();

    println!("Controller je online a naslouchá na heartbeat/#");

    // 5. HLAVNÍ ASYNCHRONNÍ SMYČKA
    loop {
        // eventloop.poll().await čeká na cokoliv, co se stane na síti
        match eventloop.poll().await {
            // Zajímá nás pouze událost "Přijata zpráva (Publish)"
            Ok(Event::Incoming(Incoming::Publish(publish_msg))) => {
                let topic = publish_msg.topic;
                let payload = publish_msg.payload;

                // Ověříme, že je to opravdu heartbeat
                if topic.starts_with("heartbeat/") {
                    // Zkusíme surové byty (payload) převést na náš silně typovaný HeartbeatMessage
                    match serde_json::from_slice::<HeartbeatMessage>(&payload) {
                        Ok(heartbeat) => {
                            println!(
                                "[\u{2713}] Heartbeat od '{}' (Stav: {:?}, Uptime: {}s)",
                                heartbeat.service_name, heartbeat.status, heartbeat.uptime_seconds
                            );

                            // Uložíme/Aktualizujeme stav služby v naší paměti
                            active_services.insert(heartbeat.service_name.clone(), heartbeat);
                        }
                        Err(e) => {
                            println!("[\u{2717}] Neplatný JSON v topicu {}: {}", topic, e);
                        }
                    }
                }
            }
            // Ignorujeme ostatní interní MQTT věci (Ping, Ack...)
            Ok(_) => {}
            // Pokud spojení spadne (např. restartuješ Mosquitto), chytneme to,
            // počkáme 3 vteřiny a smyčka pojede dál (automatický reconnect).
            Err(e) => {
                println!("[\u{26A0}] Ztráta spojení s MQTT: {:?}. Zkouším znovu...", e);
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    }
}
