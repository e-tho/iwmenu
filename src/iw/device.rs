use anyhow::{Context, Result};
use iwdrs::{device::Device as IwdDevice, modes::Mode, session::Session};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

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

        let name = device.name().await?;
        let address = device.address().await?;
        let mode = device.get_mode().await?;
        let is_powered = device.is_powered().await?;

        let station = match session.station() {
            Some(_) => match Station::new(session.clone()).await {
                Ok(v) => Some(v),
                Err(e) => {
                    let msg = format!("Failed to initialize Station: {}", e);
                    sender.send(msg).unwrap_or_else(|err| {
                        println!("Failed to send log message: {}", err);
                    });
                    None
                }
            },
            None => None,
        };

        let access_point = match session.access_point() {
            Some(_) => match AccessPoint::new(session.clone()).await {
                Ok(v) => Some(v),
                Err(e) => {
                    let msg = format!("Failed to initialize AccessPoint: {}", e);
                    sender.send(msg).unwrap_or_else(|err| {
                        println!("Failed to send log message: {}", err);
                    });
                    None
                }
            },
            None => None,
        };

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

    pub async fn set_mode(&self, mode: Mode) -> Result<()> {
        self.device.set_mode(mode).await?;
        Ok(())
    }

    pub async fn power_off(&self) -> Result<()> {
        self.device.set_power(false).await?;
        Ok(())
    }

    pub async fn power_on(&self) -> Result<()> {
        self.device.set_power(true).await?;
        Ok(())
    }

    pub async fn refresh(&mut self, sender: UnboundedSender<String>) -> Result<()> {
        self.is_powered = self.device.is_powered().await?;
        let current_mode = self.device.get_mode().await?;

        match current_mode {
            Mode::Station => match self.mode {
                Mode::Station => {
                    if let Some(station) = &mut self.station {
                        station.refresh(sender).await?;
                    }
                }
                Mode::Ap => {
                    self.access_point = None;
                    self.station = match self.session.station() {
                        Some(_) => match Station::new(self.session.clone()).await {
                            Ok(v) => Some(v),
                            Err(e) => {
                                let msg = format!("Failed to initialize Station: {}", e);
                                sender.send(msg).unwrap_or_else(|err| {
                                    println!("Failed to send log message: {}", err);
                                });
                                None
                            }
                        },
                        None => None,
                    };
                }
                _ => {}
            },
            Mode::Ap => match self.mode {
                Mode::Station => {
                    self.station = None;
                    self.access_point = match self.session.access_point() {
                        Some(_) => match AccessPoint::new(self.session.clone()).await {
                            Ok(v) => Some(v),
                            Err(e) => {
                                let msg = format!("Failed to initialize AccessPoint: {}", e);
                                sender.send(msg).unwrap_or_else(|err| {
                                    println!("Failed to send log message: {}", err);
                                });
                                None
                            }
                        },
                        None => None,
                    };
                }
                Mode::Ap => {
                    if self.access_point.is_some() {
                        self.access_point.as_mut().unwrap().refresh().await?;
                    }
                }
                _ => {}
            },
            _ => {}
        }

        self.mode = current_mode;
        Ok(())
    }
}
