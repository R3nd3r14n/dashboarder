use crate::config::MqttConfig;
use paho_mqtt as paho;
use std::time::Duration;
use tracing::info;

/// Centrální továrna na MQTT klienty.
/// Vyřeší TCP vs SSL, nastaví buffer a automatický reconnect.
pub fn connect_mqtt(config: &MqttConfig) -> (paho::Client, paho::Receiver<Option<paho::Message>>) {
    let schema = if config.host.contains("://") {
        ""
    } else if config.port == 8883 {
        "ssl://"
    } else {
        "tcp://"
    };

    let addr = format!("{}{}:{}", schema, config.host, config.port);
    let is_ssl = addr.starts_with("ssl://");

    let create_opts = paho::CreateOptionsBuilder::new()
        .server_uri(&addr)
        .client_id(&config.client_id)
        .max_buffered_messages(5000)
        .finalize();

    let cli = paho::Client::new(create_opts).expect("Chyba při vytváření MQTT klienta");
    let rx = cli.start_consuming();

    let mut conn_builder = paho::ConnectOptionsBuilder::new();
    conn_builder
        .clean_session(true)
        .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(60))
        .keep_alive_interval(Duration::from_secs(20));

    if let (Some(u), Some(p)) = (&config.username, &config.password) {
        conn_builder.user_name(u).password(p);
    }

    if is_ssl {
        let ssl_opts = paho::SslOptionsBuilder::new()
            .verify(false)
            .enable_server_cert_auth(false)
            .finalize();
        conn_builder.ssl_options(ssl_opts);
    }

    info!("Připojuji MQTT klienta [{}] na {}", config.client_id, addr);
    cli.connect(conn_builder.finalize())
        .expect("Nepodařilo se připojit k MQTT brokeru");

    (cli, rx)
}
