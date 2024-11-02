use anyhow::Result;
use futures::future::join_all;
use iwdrs::session::Session;
use rust_i18n::t;
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::mpsc::UnboundedSender, time::Duration};

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
        self.state = self.session.station().unwrap().state().await?;
        self.is_scanning = self.session.station().unwrap().is_scanning().await?;

        if self.is_scanning {
            while self.session.station().unwrap().is_scanning().await? {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        self.connected_network =
            if let Some(n) = self.session.station().unwrap().connected_network().await? {
                Some(Network::new(n.clone()).await?)
            } else {
                None
            };

        let discovered_networks = self
            .session
            .station()
            .unwrap()
            .discovered_networks()
            .await?;

        let network_futures = discovered_networks
            .into_iter()
            .map(|(n, signal)| async move {
                let network = Network::new(n.clone()).await?;
                Ok::<(Network, i16), anyhow::Error>((network, signal))
            })
            .collect::<Vec<_>>();

        let networks_results = join_all(network_futures).await;

        let mut networks = Vec::new();
        for result in networks_results {
            match result {
                Ok((network, signal)) => {
                    networks.push((network, signal));
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        self.new_networks = networks
            .iter()
            .filter(|(net, _)| net.known_network.is_none())
            .cloned()
            .collect();

        self.known_networks = networks
            .iter()
            .filter(|(net, _)| net.known_network.is_some())
            .cloned()
            .collect();

        if let Some(station_diagnostic) = self.session.station_diagnostic() {
            if let Ok(diagnostic) = station_diagnostic.get().await {
                self.diagnostic = diagnostic;
            }
        }

        Ok(())
    }

    pub async fn scan(&self) -> Result<()> {
        let iwd_station = self.session.station().unwrap();
        iwd_station.scan().await.map_err(|e| e.into())
    }

    pub async fn disconnect(
        &mut self,
        sender: UnboundedSender<String>,
        notification_manager: Arc<NotificationManager>,
    ) -> Result<()> {
        let iwd_station = self.session.station().unwrap();
        match iwd_station.disconnect().await {
            Ok(_) => {
                let msg = t!(
                    "notifications.station.disconnected_from_network",
                    network_name = self.connected_network.as_ref().unwrap().name
                );
                sender
                    .send(msg.to_string())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                try_send_notification!(
                    notification_manager,
                    None,
                    Some(msg.to_string()),
                    None,
                    None
                );
            }
            Err(e) => {
                let msg = t!(
                    "notifications.station.error_disconnecting",
                    error_message = e.to_string()
                );
                sender
                    .send(msg.to_string())
                    .unwrap_or_else(|err| println!("Failed to send message: {}", err));
                try_send_notification!(
                    notification_manager,
                    None,
                    Some(msg.to_string()),
                    None,
                    None
                );
            }
        }
        Ok(())
    }
}
