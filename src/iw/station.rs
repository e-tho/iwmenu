use anyhow::Result;
use futures::future::join_all;
use iwdrs::session::Session;
use notify_rust::Timeout;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    sync::mpsc::UnboundedSender,
    time::{sleep, Duration},
};

use crate::{iw::network::Network, notification::NotificationManager};

#[derive(Debug, Clone)]
pub struct Station {
    pub session: Arc<Session>,
    pub state: String,
    pub is_scanning: bool,
    pub connected_network: Option<Network>,
    pub new_networks: Vec<(Network, i16)>,
    pub known_networks: Vec<(Network, i16)>,
    pub diagnostic: HashMap<String, String>,
}

impl Station {
    pub async fn new(session: Arc<Session>) -> Result<Self> {
        let iwd_station = session.station().unwrap();
        let iwd_station_diagnostic = session.station_diagnostic();

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

        let mut diagnostic: HashMap<String, String> = HashMap::new();
        if let Some(station_diagnostic) = iwd_station_diagnostic {
            if let Ok(d) = station_diagnostic.get().await {
                diagnostic = d;
            }
        }

        Ok(Self {
            session,
            state,
            is_scanning,
            connected_network,
            new_networks,
            known_networks,
            diagnostic,
        })
    }

    pub async fn refresh(&mut self) -> Result<()> {
        let iwd_station = self.session.station().unwrap();
        let iwd_station_diagnostic = self.session.station_diagnostic();

        let state = iwd_station.state().await?;
        let is_scanning = iwd_station.is_scanning().await?;

        if is_scanning {
            while iwd_station.is_scanning().await? {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

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

        if let Some(station_diagnostic) = iwd_station_diagnostic {
            if let Ok(d) = station_diagnostic.get().await {
                self.diagnostic = d;
            }
        }

        Ok(())
    }

    pub async fn scan(
        &self,
        sender: UnboundedSender<String>,
        notification_manager: Arc<NotificationManager>,
    ) -> Result<()> {
        let iwd_station = self.session.station().unwrap();

        if iwd_station.is_scanning().await? {
            sender
                .send("Scan already in progress, waiting...".to_string())
                .unwrap_or_else(|err| println!("Failed to send message: {}", err));

            notification_manager.send_notification(
                None,
                Some("Scan already in progress, waiting...".to_string()),
                None,
                None,
            );

            return Ok(());
        }

        let handle = match iwd_station.scan().await {
            Ok(_) => {
                sender
                    .send("Start Scanning".to_string())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));

                Some(notification_manager.send_notification(
                    None,
                    Some("Wi-Fi scan in progress".to_string()),
                    None,
                    Some(Timeout::Never),
                ))
            }
            Err(e) => {
                sender
                    .send(e.to_string())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));

                notification_manager.send_notification(None, Some(e.to_string()), None, None);

                return Err(e.into());
            }
        };

        loop {
            sleep(Duration::from_millis(500)).await;
            if !iwd_station.is_scanning().await? {
                break;
            }
        }

        if let Some(handle) = handle {
            handle.close();
        }

        sender
            .send("Scan completed".to_string())
            .unwrap_or_else(|err| println!("Failed to send message: {}", err));

        Ok(())
    }

    pub async fn disconnect(
        &mut self,
        sender: UnboundedSender<String>,
        notification_manager: Arc<NotificationManager>,
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
                notification_manager.send_notification(
                    None,
                    Some(msg.clone()),
                    None,
                    Some(Timeout::Milliseconds(3000)),
                );
            }
            Err(e) => {
                let msg = e.to_string();
                sender
                    .send(msg.clone())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                notification_manager.send_notification(
                    None,
                    Some(msg.clone()),
                    None,
                    Some(Timeout::Milliseconds(3000)),
                );
            }
        }
        Ok(())
    }
}
