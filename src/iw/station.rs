use anyhow::Result;
use futures::future::join_all;
use iwdrs::session::Session;
use notify_rust::Timeout;
use std::sync::Arc;
use tokio::{
    sync::mpsc::UnboundedSender,
    time::{sleep, Duration},
};

use crate::iw::network::Network;

#[derive(Debug, Clone)]
pub struct Station {
    pub session: Arc<Session>,
    pub state: String,
    pub is_scanning: bool,
    pub connected_network: Option<Network>,
    pub new_networks: Vec<(Network, i16)>,
    pub known_networks: Vec<(Network, i16)>,
}

impl Station {
    pub async fn new(session: Arc<Session>) -> Result<Self> {
        let iwd_station = session.station().unwrap();

        let state = iwd_station.state().await?;
        let connected_network = {
            if let Some(n) = iwd_station.connected_network().await? {
                let network = Network::new(n.clone()).await?;
                Some(network)
            } else {
                None
            }
        };

        let is_scanning = iwd_station.is_scanning().await?;
        let discovered_networks = iwd_station.discovered_networks().await?;
        let networks = {
            let collected_futures = discovered_networks
                .iter()
                .map(|(n, signal)| async move {
                    match Network::new(n.clone()).await {
                        Ok(network) => Ok((network, signal.to_owned())),
                        Err(e) => Err(e),
                    }
                })
                .collect::<Vec<_>>();
            let results = join_all(collected_futures).await;
            results
                .into_iter()
                .filter_map(Result::ok)
                .collect::<Vec<(Network, i16)>>()
        };

        let new_networks: Vec<(Network, i16)> = networks
            .clone()
            .into_iter()
            .filter(|(net, _signal)| net.known_network.is_none())
            .collect();

        let known_networks: Vec<(Network, i16)> = networks
            .into_iter()
            .filter(|(net, _signal)| net.known_network.is_some())
            .collect();

        Ok(Self {
            session,
            state,
            is_scanning,
            connected_network,
            new_networks,
            known_networks,
        })
    }

    pub async fn refresh(&mut self) -> Result<()> {
        let iwd_station = self.session.station().unwrap();

        let state = iwd_station.state().await?;
        let is_scanning = iwd_station.is_scanning().await?;
        let connected_network = {
            if let Some(n) = iwd_station.connected_network().await? {
                let network = Network::new(n.clone()).await?;
                Some(network.to_owned())
            } else {
                None
            }
        };
        let discovered_networks = iwd_station.discovered_networks().await?;
        let networks = {
            let collected_futures = discovered_networks
                .iter()
                .map(|(n, signal)| async move {
                    match Network::new(n.clone()).await {
                        Ok(network) => Ok((network, signal.to_owned())),
                        Err(e) => Err(e),
                    }
                })
                .collect::<Vec<_>>();
            let results = join_all(collected_futures).await;
            results
                .into_iter()
                .filter_map(Result::ok)
                .collect::<Vec<(Network, i16)>>()
        };

        let new_networks: Vec<(Network, i16)> = networks
            .clone()
            .into_iter()
            .filter(|(net, _signal)| net.known_network.is_none())
            .collect();

        let known_networks: Vec<(Network, i16)> = networks
            .into_iter()
            .filter(|(net, _signal)| net.known_network.is_some())
            .collect();

        self.state = state;
        self.is_scanning = is_scanning;
        self.connected_network = connected_network;
        self.new_networks = new_networks;
        self.known_networks = known_networks;

        Ok(())
    }

    pub async fn scan(
        &self,
        sender: UnboundedSender<String>,
        notification_sender: UnboundedSender<(
            Option<String>,
            Option<String>,
            Option<String>,
            Option<Timeout>,
        )>,
    ) -> Result<()> {
        let iwd_station = self.session.station().unwrap();

        if iwd_station.is_scanning().await? {
            sender
                .send("Scan already in progress, waiting...".to_string())
                .unwrap_or_else(|err| println!("Failed to send message: {}", err));

            notification_sender
                .send((
                    None,
                    Some("Scan already in progress, waiting...".to_string()),
                    None,
                    None,
                ))
                .unwrap_or_else(|err| println!("Failed to send notification: {}", err));

            return Ok(());
        }

        match iwd_station.scan().await {
            Ok(_) => {
                sender
                    .send("Start Scanning".to_string())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));

                notification_sender
                    .send((None, Some("Starting Wi-Fi scan...".to_string()), None, None))
                    .unwrap_or_else(|err| println!("Failed to send notification: {}", err));
            }
            Err(e) => {
                sender
                    .send(e.to_string())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));

                notification_sender
                    .send((None, Some(e.to_string()), None, None))
                    .unwrap_or_else(|err| println!("Failed to send notification: {}", err));

                return Err(e.into());
            }
        }

        loop {
            sleep(Duration::from_millis(500)).await;
            if !iwd_station.is_scanning().await? {
                break;
            }
        }

        sender
            .send("Scan completed".to_string())
            .unwrap_or_else(|err| println!("Failed to send message: {}", err));

        Ok(())
    }

    pub async fn disconnect(
        &mut self,
        sender: UnboundedSender<String>,
        notification_sender: UnboundedSender<(
            Option<String>,
            Option<String>,
            Option<String>,
            Option<Timeout>,
        )>,
    ) -> Result<()> {
        let iwd_station = self.session.station().unwrap();
        match iwd_station.disconnect().await {
            Ok(_) => {
                let msg = format!(
                    "Disconnected from {}",
                    self.connected_network.as_ref().unwrap().name
                );
                sender
                    .send(msg.clone())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                notification_sender
                    .send((None, Some(msg.clone()), None, None))
                    .unwrap_or_else(|err| println!("Failed to send notification: {}", err));
            }
            Err(e) => {
                let msg = e.to_string();
                sender
                    .send(msg)
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));
            }
        }

        self.refresh().await?;

        Ok(())
    }
}
