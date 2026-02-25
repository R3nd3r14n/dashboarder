// Importujeme potřebné věci z naší vlastní sdílené knihovny (common_models).
use common_models::{
    connect_mqtt, init_logging, load_config, paho_mqtt as mqtt, DashboardMessage, HeartbeatMessage,
    MessageType, MqttConfig, ServiceStatus,
};
use std::time::Duration;
// Tracing pro výpisy do konzole
use tracing::{error, info, warn};

fn main() {
    // 1. INICIALIZACE SYSTÉMU
    init_logging();

    // 2. NAČTENÍ KONFIGURACE (z mqtt_bridge_rpi.toml)
    let full_config: serde_json::Value =
        load_config("mqtt_bridge_rpi").expect("Konfigurace nebyla nalezena");

    let ext_cfg: MqttConfig =
        serde_json::from_value(full_config["external"].clone()).expect("Chyba v sekci [external]");
    let int_cfg: MqttConfig =
        serde_json::from_value(full_config["internal"].clone()).expect("Chyba v sekci [internal]");

    let sub_topic = ext_cfg
        .topic
        .clone()
        .unwrap_or_else(|| "msh/internal_temp/#".to_string());
    let pub_topic = int_cfg
        .topic
        .clone()
        .unwrap_or_else(|| "vnitrni/senzory/data".to_string());

    // Změněno z portabo na roomtemp, ať máme v heartbeatech pořádek
    let service_name = "mqtt-bridge-rpi";

    // 3. PŘIPOJENÍ K BROKERŮM (MAGIE SDÍLENÉ KNIHOVNY)
    // A) Připojení "ven" (v tomto případě na kanál senzoru)
    let (ext_cli, rx) = connect_mqtt(&ext_cfg);

    // B) Připojení "dovnitř" (kam posíláme vyčištěná data)
    let (int_cli, _int_rx) = connect_mqtt(&int_cfg);

    // 4. HEARTBEAT (TLUKOT SRDCE SLUŽBY)
    let hb_cli = int_cli.clone();
    let hb_name = service_name.to_string();

    std::thread::spawn(move || {
        let mut uptime = 0;
        loop {
            let hb = HeartbeatMessage::new(&hb_name, ServiceStatus::Ok, uptime);
            if let Ok(payload) = serde_json::to_vec(&hb) {
                let topic = format!("heartbeat/{}", hb_name);
                let msg = mqtt::Message::new(topic, payload, 0);
                let _ = hb_cli.publish(msg);
            }
            std::thread::sleep(Duration::from_secs(15));
            uptime += 15;
        }
    });

    // 5. ODBĚR DAT A HLAVNÍ SMYČKA (RUTINA MOSTU)
    ext_cli
        .subscribe(&sub_topic, 0)
        .expect("Chyba při odebírání (subscribe)");
    info!(
        "Most aktivní. Přelévám a parsuji data: {} -> {}",
        sub_topic, pub_topic
    );

    // SMYČKA ČEKAJÍCÍ NA DATA
    for msg_opt in rx.iter() {
        if let Some(msg) = msg_opt {
            // A) NORMALIZACE ZDROJE
            // Z "/msh/internal_temp/ds2" udělá "mqtt-bridge--msh-internal_temp-ds2"
            let source_id = format!("mqtt-bridge-{}", msg.topic().replace("/", "-"));

            // REFLEKTOR 1: Vypíšeme, že se něco děje, abychom nečučeli na černou obrazovku
            info!("📥 Dorazila zpráva na téma: {}", msg.topic());

            // B) PŘEVOD BYTŮ NA TEXT
            // MQTT zpráva (msg.payload) jsou vždy jen surové nuly a jedničky (byty).
            // from_utf8_lossy z toho udělá normální text (String).
            // "Lossy" znamená, že když tam bude poškozený znak, program nespadne, ale nahradí ho otazníkem.
            let payload_str = String::from_utf8_lossy(msg.payload());

            // C) PARSOVÁNÍ TEXTU NA ČÍSLO
            // .trim() ořízne případné neviditelné mezery nebo Entery (\n), které tam senzor mohl přidat.
            // .parse::<f64>() to zkusí převést na desetinné číslo (float).
            match payload_str.trim().parse::<f64>() {
                Ok(temperature) => {
                    // D) VÝROBA NOVÉHO JSONu
                    // Tady je to kouzlo. Máme čisté číslo (např. 22.5). My z něj pomocí makra json!
                    // vyrobíme validní JSON objekt: {"temperature": 22.5}
                    let json_value = serde_json::json!({ "temperature": temperature });

                    // E) ZABALENÍ DO NAŠEHO STANDARDU
                    let mut d_msg = DashboardMessage::new(MessageType::Value);
                    d_msg.set_value(json_value); // Vložíme náš nově vytvořený JSON
                    d_msg.set_source(&source_id);

                    // F) ODESLÁNÍ
                    if let Ok(serialized) = serde_json::to_vec(&d_msg) {
                        let out_msg = mqtt::Message::new(&pub_topic, serialized, 0);
                        if let Err(e) = int_cli.publish(out_msg) {
                            error!("❌ Chyba při přeposílání zprávy pro {}: {:?}", source_id, e);
                        } else {
                            // REFLEKTOR 2: Vše klaplo
                            info!("✅ Přeposláno! {} = {} °C", source_id, temperature);
                        }
                    }
                }
                Err(e) => {
                    // REFLEKTOR CHYB: Pokud senzor pošle text "Ahoj" místo čísla, nespadne to, ale řekne ti proč!
                    warn!(
                        "⚠️ Nelze převést na číslo. Surová data: '{}' | Chyba: {}",
                        payload_str, e
                    );
                }
            }
        } else {
            warn!("Buffer zpráv je prázdný nebo spojení selhalo. Čekám na obnovu...");
        }
    }
}
