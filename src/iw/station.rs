use anyhow::{anyhow, Result};
use futures_util::future::join_all;
use iwdrs::{session::Session, station::State};
use std::sync::Arc;
use tokio::time::Duration;

use crate::iw::network::Network;

#[derive(Debug, Clone)]
pub struct Station {
    pub session: Arc<Session>,
    pub state: State,
    pub is_scanning: bool,
    pub connected_network: Option<Network>,
    pub new_networks: Vec<(Network, i16)>,
    pub known_networks: Vec<(Network, i16)>,
    pub diagnostic: Option<iwdrs::station::diagnostics::ActiveStationDiagnostics>,
}

impl Station {
    pub async fn new(session: Arc<Session>) -> Result<Self> {
        let stations = session.stations().await?;
        let iwd_station = stations
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Failed to retrieve station from session"))?;

        let iwd_station_diagnostic = session
            .stations_diagnostics()
            .await
            .ok()
            .and_then(|v| v.into_iter().next());

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
                        .map_err(|e| anyhow!("Failed to create network: {e:?}"))
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

        let diagnostic = if let Some(station_diagnostic) = iwd_station_diagnostic {
            station_diagnostic.get().await.ok()
        } else {
            None
        };

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
        let stations = self.session.stations().await?;
        let station = stations
            .into_iter()
            .next()
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
                    .map_err(|e| anyhow!("Failed to process network: {e:?}"))
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

        if let Ok(diagnostics) = self.session.stations_diagnostics().await {
            if let Some(diagnostic) = diagnostics.into_iter().next() {
                self.diagnostic = diagnostic.get().await.ok();
            }
        }

        Ok(())
    }

    pub async fn scan(&self) -> Result<()> {
        let stations = self.session.stations().await?;
        let station = stations
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Failed to retrieve station from session"))?;

        station
            .scan()
            .await
            .map_err(|e| anyhow!("Failed to start scan: {e:?}"))
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        let stations = self.session.stations().await?;
        let station = stations
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Failed to retrieve station from session"))?;

        station
            .disconnect()
            .await
            .map_err(|e| anyhow!("Failed to disconnect: {e:?}"))
    }
}
