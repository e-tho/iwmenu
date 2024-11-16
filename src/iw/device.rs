use anyhow::{Context, Result};
use iwdrs::{device::Device as IwdDevice, modes::Mode, session::Session};
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::iw::{access_point::AccessPoint, station::Station};

#[derive(Debug, Clone)]
pub struct Device {
    session: Arc<Session>,
    pub device: IwdDevice,
    pub name: String,
    pub address: String,
    pub mode: Mode,
    pub is_powered: bool,
    pub station: Option<Station>,
    pub access_point: Option<AccessPoint>,
}

impl Device {
    pub async fn new(session: Arc<Session>, sender: UnboundedSender<String>) -> Result<Self> {
        let device = session.device().context("No device found")?;

        let name = device
            .name()
            .await
            .context("Failed to retrieve device name")?;
        let address = device
            .address()
            .await
            .context("Failed to retrieve device address")?;
        let mode = device
            .get_mode()
            .await
            .context("Failed to retrieve device mode")?;
        let is_powered = device
            .is_powered()
            .await
            .context("Failed to check if the device is powered")?;

        let station = Self::initialize_station(session.clone(), sender.clone()).await;
        let access_point = Self::initialize_access_point(session.clone(), sender.clone()).await;

        Ok(Self {
            session,
            device,
            name,
            address,
            mode,
            is_powered,
            station,
            access_point,
        })
    }

    async fn initialize_station(
        session: Arc<Session>,
        sender: UnboundedSender<String>,
    ) -> Option<Station> {
        match session.station() {
            Some(_) => match Station::new(session).await {
                Ok(station) => Some(station),
                Err(e) => {
                    try_send_log!(sender, format!("Failed to initialize Station: {}", e));
                    None
                }
            },
            None => None,
        }
    }

    async fn initialize_access_point(
        session: Arc<Session>,
        sender: UnboundedSender<String>,
    ) -> Option<AccessPoint> {
        match session.access_point() {
            Some(_) => match AccessPoint::new(session).await {
                Ok(access_point) => Some(access_point),
                Err(e) => {
                    try_send_log!(sender, format!("Failed to initialize AccessPoint: {}", e));
                    None
                }
            },
            None => None,
        }
    }

    pub async fn set_mode(&self, mode: Mode) -> Result<()> {
        self.device
            .set_mode(mode)
            .await
            .context("Failed to set device mode")
    }

    pub async fn power_off(&self) -> Result<()> {
        self.device
            .set_power(false)
            .await
            .context("Failed to power off the device")
    }

    pub async fn power_on(&self) -> Result<()> {
        self.device
            .set_power(true)
            .await
            .context("Failed to power on the device")
    }

    pub async fn refresh(&mut self) -> Result<()> {
        self.is_powered = self
            .device
            .is_powered()
            .await
            .context("Failed to check if the device is powered")?;

        let current_mode = self
            .device
            .get_mode()
            .await
            .context("Failed to retrieve current device mode")?;

        self.update_mode(current_mode.clone()).await?;

        self.mode = current_mode;
        Ok(())
    }

    async fn update_mode(&mut self, current_mode: Mode) -> Result<()> {
        match current_mode {
            Mode::Station => {
                if self.mode == Mode::Station {
                    if let Some(station) = &mut self.station {
                        station
                            .refresh()
                            .await
                            .context("Failed to refresh Station")?;
                    }
                } else {
                    self.access_point = None;
                    self.station =
                        Self::initialize_station(self.session.clone(), unbounded_channel().0).await;
                }
            }
            Mode::Ap => {
                if self.mode == Mode::Ap {
                    if let Some(access_point) = &mut self.access_point {
                        access_point
                            .refresh()
                            .await
                            .context("Failed to refresh AccessPoint")?;
                    }
                } else {
                    self.station = None;
                    self.access_point =
                        Self::initialize_access_point(self.session.clone(), unbounded_channel().0)
                            .await;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
