use crate::dataprocessor::GrowattData;
use crate::{layouts, ProxyError};
use log;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpSocket, TcpStream};

pub struct GrowattProxy {
    address: String,
    growatt_address: String,
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
    pub fn new(proxy_address: &str, growatt_address: &str) -> GrowattProxy {
        GrowattProxy {
            address: String::from(proxy_address),
            growatt_address: String::from(growatt_address),
        }
    }

    pub async fn run(self) -> Result<(), ProxyError> {
        let listener = TcpListener::bind(&self.address).await?;

        loop {
            let (mut socket, _) = listener.accept().await?;
            let growatt_addr = self.growatt_address.to_owned();

            tokio::spawn(async move {
                log::info!("Inverter connected");
                let mut buf = vec![0; 4096];
                let mut growatt_buf = vec![0; 4096];

                let mut growatt_data = Vec::new();

                if let Ok(mut forwarder) = GrowattForwarder::new(growatt_addr).await {
                    loop {
                        tokio::select! {
                            Ok(n) = socket.read(&mut buf) =>  {
                                log::info!("Data from inverter: {}", n);
                                if n == 0 {
                                    return;
                                }

                                if n > 128 {
                                    if let Ok(data) = GrowattData::from_buffer(&mut growatt_data, &layouts::t065004x()) {
                                        // Do something with the decoded data
                                    }
                                }

                                log::info!("Got inverter data: size {}", n);
                                growatt_data.extend_from_slice(&buf[..n]);

                                // Forward data to the growatt server if we are connected
                                if let Err(err) = forwarder.stream.write_all(&buf[..n]).await {
                                    log::warn!("Failed to forward data to Growatt server: {err}");
                                    return;
                                }
                            }

                            Ok(n) = forwarder.stream.read(&mut growatt_buf) => {
                                log::info!("Response from growatt: {}", n);
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
