use anyhow::{anyhow, Result};
use futures::FutureExt;
use iwdrs::{agent::Agent, session::Session};
use std::sync::{
    atomic::{AtomicBool, Ordering::Relaxed},
    Arc,
};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    Mutex,
};

pub struct AgentManager {
    session: Arc<Session>,
    authentication_required: Arc<AtomicBool>,
    passkey_sender: UnboundedSender<String>,
    cancel_signal_sender: UnboundedSender<()>,
}

impl AgentManager {
    pub async fn new() -> Result<Self> {
        let session = Arc::new(Session::new().await?);

        let (passkey_sender, passkey_receiver) = unbounded_channel::<String>();
        let (cancel_signal_sender, cancel_signal_receiver) = unbounded_channel::<()>();

        let passkey_receiver = Arc::new(Mutex::new(passkey_receiver));
        let cancel_signal_receiver = Arc::new(Mutex::new(cancel_signal_receiver));

        let authentication_required = Arc::new(AtomicBool::new(false));

        let agent = {
            let authentication_required_clone = authentication_required.clone();
            let passkey_receiver_clone = passkey_receiver.clone();
            let cancel_signal_receiver_clone = cancel_signal_receiver.clone();

            Agent {
                request_passphrase_fn: Box::new(move || {
                    let authentication_required = authentication_required_clone.clone();
                    let passkey_receiver = passkey_receiver_clone.clone();
                    let cancel_signal_receiver = cancel_signal_receiver_clone.clone();

                    async move {
                        let mut rx_key = passkey_receiver.lock().await;
                        let mut rx_cancel = cancel_signal_receiver.lock().await;

                        request_confirmation(authentication_required, &mut rx_key, &mut rx_cancel)
                            .await
                    }
                    .boxed()
                }),
            }
        };

        session.register_agent(agent).await?;

        Ok(Self {
            session,
            authentication_required,
            passkey_sender,
            cancel_signal_sender,
        })
    }

    pub fn session(&self) -> Arc<Session> {
        self.session.clone()
    }

    pub fn send_passkey(&self, passkey: String) -> Result<()> {
        self.passkey_sender.send(passkey)?;
        self.authentication_required.store(false, Relaxed);
        Ok(())
    }

    pub fn cancel_auth(&self) -> Result<()> {
        self.cancel_signal_sender.send(())?;
        self.authentication_required.store(false, Relaxed);
        Ok(())
    }
}

pub async fn request_confirmation(
    authentication_required: Arc<AtomicBool>,
    rx_key: &mut UnboundedReceiver<String>,
    rx_cancel: &mut UnboundedReceiver<()>,
) -> Result<String, Box<dyn std::error::Error>> {
    authentication_required.store(true, Relaxed);

    let result = tokio::select! {
        received_key = rx_key.recv() => {
            match received_key {
                Some(key) => Ok(key),
                None => Err(Box::<dyn std::error::Error>::from(anyhow!("Failed to receive the key"))),
            }
        }
        received_cancel = rx_cancel.recv() => {
            match received_cancel {
                Some(_) => Err(Box::<dyn std::error::Error>::from(anyhow!("Operation Canceled"))),
                None => Err(Box::<dyn std::error::Error>::from(anyhow!("Failed to receive cancel signal"))),
            }
        }
    };

    authentication_required.store(false, Relaxed);
    result
}
