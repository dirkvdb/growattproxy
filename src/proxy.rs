use crate::dataprocessor::GrowattData;
use crate::mqtt::{self, MqttConfig};
use crate::ProxyError;
use log;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpSocket, TcpStream};

pub struct GrowattProxyConfig {
    pub listen_address: String,
    pub growatt_address: String,
    pub mqtt_address: Option<String>,
    pub mqtt_port: u16,
}

pub struct GrowattProxy {
    address: String,
    growatt_address: String,
    mqtt_config: Option<MqttConfig>,
}

struct GrowattForwarder {
    pub stream: TcpStream,
}

impl GrowattForwarder {
    pub async fn new(address: String) -> Result<GrowattForwarder, ProxyError> {
        let addr = address.parse()?;

        let socket = TcpSocket::new_v4()?;
        let stream = socket.connect(addr).await?;

        Ok(GrowattForwarder { stream })
    }
}

impl GrowattProxy {
    pub fn new(cfg: GrowattProxyConfig) -> GrowattProxy {
        let mqtt_config;
        if let Some(addr) = cfg.mqtt_address {
            log::info!("MQTT configuration: {}:{}", addr, cfg.mqtt_port);
            mqtt_config = Some(MqttConfig {
                server: addr,
                port: cfg.mqtt_port,
            });
        } else {
            mqtt_config = None;
        }

        GrowattProxy {
            address: String::from(cfg.listen_address),
            growatt_address: String::from(cfg.growatt_address),
            mqtt_config,
        }
    }

    pub async fn run(self) -> Result<(), ProxyError> {
        let listener = TcpListener::bind(&self.address).await?;

        loop {
            let (mut socket, _) = listener.accept().await?;
            socket.set_nodelay(true)?;

            let growatt_addr = self.growatt_address.to_owned();

            let mqtt_config = self.mqtt_config.to_owned();

            tokio::spawn(async move {
                log::info!("Inverter connected");
                let mut buf = vec![0; 4096];
                let mut growatt_buf = vec![0; 4096];

                let mut growatt_data = Vec::new();

                if let Ok(mut forwarder) = GrowattForwarder::new(growatt_addr).await {
                    loop {
                        tokio::select! {
                            Ok(n) = socket.read(&mut buf) =>  {
                                if n == 0 {
                                    return;
                                }

                                log::debug!("Got inverter data: size {}", n);
                                growatt_data.clear();
                                growatt_data.extend_from_slice(&buf[..n]);

                                if n > 128 {
                                    match GrowattData::from_buffer_auto_detect_layout(&mut growatt_data) {
                                        Ok(data) => {
                                            if data.has_data() {
                                                if let Some(cfg) = mqtt_config.as_ref() {
                                                    log::info!("Growatt data: [#{}] {} -> {} (Buffered: {})", data.packet_index(), data.layout(), data.layout_spec, data.is_buffered());
                                                    if let Err(err) = mqtt::publish_data(&data, &cfg).await {
                                                        log::warn!("Failed to publish MQTT data: {err}");
                                                    }
                                                }
                                            } else {
                                                log::info!("Growatt data ignored: [#{}] {} -> {} (Buffered: {})", data.packet_index(), data.layout(), data.layout_spec, data.is_buffered());
                                            }
                                        }
                                        Err(err) => log::warn!("Invalid growatt data: {}", err)
                                    }
                                }

                                // Forward data to the growatt server if we are connected
                                if let Err(err) = forwarder.stream.write_all(&buf[..n]).await {
                                    log::warn!("Failed to forward data to Growatt server: {err}");
                                    return;
                                }
                            }

                            Ok(n) = forwarder.stream.read(&mut growatt_buf) => {
                                log::debug!("Response from growatt: {}", n);
                                if n == 0 {
                                    return;
                                }

                                if let Err(err) = socket.write_all(&growatt_buf[..n]).await {
                                    log::warn!("Failed to forward response from Growatt server: {err}");
                                    return;
                                }
                            }
                        }
                    }
                } else {
                    log::warn!("Failed to connect to growatt server, data will not be forwarded");
                }
            });
        }
    }
}
