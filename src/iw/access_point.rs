use anyhow::Result;
use iwdrs::session::Session;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AccessPoint {
    session: Arc<Session>,
    pub has_started: bool,
    pub name: Option<String>,
    pub frequency: Option<u32>,
    pub is_scanning: Option<bool>,
    pub supported_ciphers: Option<Vec<String>>,
    pub used_cipher: Option<String>,
    pub connected_devices: Vec<String>,
    pub ssid: String,
    pub psk: String,
}

impl AccessPoint {
    pub async fn new(session: Arc<Session>) -> Result<Self> {
        let iwd_access_point = session.access_point().unwrap();
        let iwd_access_point_diagnotic = session.access_point_diagnostic();

        let has_started = iwd_access_point.has_started().await?;
        let name = iwd_access_point.name().await?;
        let frequency = iwd_access_point.frequency().await?;
        let is_scanning = iwd_access_point.is_scanning().await.ok();
        let supported_ciphers = iwd_access_point.pairwise_ciphers().await?;
        let used_cipher = iwd_access_point.group_cipher().await?;

        let connected_devices = {
            if let Some(d) = iwd_access_point_diagnotic {
                match d.get().await {
                    Ok(diagnostic) => diagnostic
                        .iter()
                        .map(|v| v["Address"].clone().trim_matches('"').to_string())
                        .collect(),
                    Err(_) => Vec::new(),
                }
            } else {
                Vec::new()
            }
        };

        Ok(Self {
            session,
            has_started,
            name,
            frequency,
            is_scanning,
            supported_ciphers,
            used_cipher,
            connected_devices,
            ssid: String::new(),
            psk: String::new(),
        })
    }

    pub async fn refresh(&mut self) -> Result<()> {
        let iwd_access_point = self.session.access_point().unwrap();
        let iwd_access_point_diagnotic = self.session.access_point_diagnostic();

        self.has_started = iwd_access_point.has_started().await?;
        self.name = iwd_access_point.name().await?;
        self.frequency = iwd_access_point.frequency().await?;
        self.is_scanning = iwd_access_point.is_scanning().await.ok();
        self.supported_ciphers = iwd_access_point.pairwise_ciphers().await?;
        self.used_cipher = iwd_access_point.group_cipher().await?;

        if let Some(d) = iwd_access_point_diagnotic {
            if let Ok(diagnostic) = d.get().await {
                self.connected_devices = diagnostic
                    .iter()
                    .map(|v| v["Address"].clone().trim_matches('"').to_string())
                    .collect()
            }
        }

        Ok(())
    }

    pub async fn scan(&self) -> Result<()> {
        let iwd_access_point = self.session.access_point().unwrap();
        iwd_access_point.scan().await?;
        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        let iwd_access_point = self.session.access_point().unwrap();
        iwd_access_point.start(&self.ssid, &self.psk).await?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let iwd_access_point = self.session.access_point().unwrap();
        iwd_access_point.stop().await?;
        Ok(())
    }

    pub fn set_ssid(&mut self, ssid: String) {
        self.ssid = ssid;
    }

    pub fn set_psk(&mut self, psk: String) {
        self.psk = psk;
    }
}
