use anyhow::{anyhow, Result};
use futures::future::join_all;
use iwdrs::session::Session;
use std::{collections::HashMap, sync::Arc};
use tokio::time::Duration;

use crate::iw::network::Network;

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
        let iwd_station = session
            .station()
            .ok_or_else(|| anyhow!("Failed to retrieve station from session"))?;

        let iwd_station_diagnostic = session.station_diagnostic();

        let state = iwd_station.state().await?;

        let connected_network = if let Some(n) = iwd_station.connected_network().await? {
            Some(Network::new(n.clone()).await?)
        } else {
            None
        };

        let is_scanning = iwd_station.is_scanning().await?;

        let discovered_networks = iwd_station.discovered_networks().await?;

        let networks = {
            let collected_futures = discovered_networks
                .iter()
                .map(|(n, signal)| async move {
                    Network::new(n.clone())
                        .await
                        .map(|network| (network, *signal))
                        .map_err(|e| anyhow!("Failed to create network: {:?}", e))
                })
                .collect::<Vec<_>>();

            join_all(collected_futures)
                .await
                .into_iter()
                .filter_map(Result::ok)
                .collect::<Vec<(Network, i16)>>()
        };

        let new_networks: Vec<(Network, i16)> = networks
            .iter()
            .filter(|(net, _)| net.known_network.is_none())
            .cloned()
            .collect();

        let known_networks: Vec<(Network, i16)> = networks
            .into_iter()
            .filter(|(net, _)| net.known_network.is_some())
            .collect();

        let mut diagnostic = HashMap::new();
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
        let station = self
            .session
            .station()
            .ok_or_else(|| anyhow!("Failed to retrieve station from session"))?;

        self.state = station.state().await?;
        self.is_scanning = station.is_scanning().await?;

        if self.is_scanning {
            while station.is_scanning().await? {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        self.connected_network = if let Some(n) = station.connected_network().await? {
            Some(Network::new(n.clone()).await?)
        } else {
            None
        };

        let discovered_networks = station.discovered_networks().await?;

        let network_futures = discovered_networks
            .into_iter()
            .map(|(n, signal)| async move {
                Network::new(n.clone())
                    .await
                    .map(|network| (network, signal))
                    .map_err(|e| anyhow!("Failed to process network: {:?}", e))
            })
            .collect::<Vec<_>>();

        let networks = join_all(network_futures)
            .await
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<(Network, i16)>>();

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

        if let Some(diagnostic) = self.session.station_diagnostic() {
            if let Ok(d) = diagnostic.get().await {
                self.diagnostic = d;
            }
        }

        Ok(())
    }

    pub async fn scan(&self) -> Result<()> {
        let station = self
            .session
            .station()
            .ok_or_else(|| anyhow!("Failed to retrieve station from session"))?;

        station
            .scan()
            .await
            .map_err(|e| anyhow!("Failed to start scan: {:?}", e))
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        let station = self
            .session
            .station()
            .ok_or_else(|| anyhow!("Failed to retrieve station from session"))?;

        station
            .disconnect()
            .await
            .map_err(|e| anyhow!("Failed to disconnect: {:?}", e))
    }
}
