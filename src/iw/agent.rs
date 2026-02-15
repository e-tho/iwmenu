use anyhow::{anyhow, Context, Result};
use iwdrs::{
    agent::{Agent, CancellationReason},
    error::agent::Canceled,
    network::Network,
    session::Session,
};
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
        let session = Arc::new(
            Session::new()
                .await
                .context("Failed to initialize a new session")?,
        );

        let (passkey_sender, passkey_receiver) = unbounded_channel::<String>();
        let (cancel_signal_sender, cancel_signal_receiver) = unbounded_channel::<()>();

        let passkey_receiver = Arc::new(Mutex::new(passkey_receiver));
        let cancel_signal_receiver = Arc::new(Mutex::new(cancel_signal_receiver));
        let authentication_required = Arc::new(AtomicBool::new(false));

        let agent = CustomAgent {
            authentication_required: authentication_required.clone(),
            passkey_receiver: passkey_receiver.clone(),
            cancel_signal_receiver: cancel_signal_receiver.clone(),
        };

        session
            .register_agent(agent)
            .await
            .context("Failed to register agent")?;

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
        self.passkey_sender
            .send(passkey)
            .context("Failed to send passkey")?;

        self.authentication_required.store(false, Relaxed);
        Ok(())
    }

    pub fn cancel_auth(&self) -> Result<()> {
        self.cancel_signal_sender
            .send(())
            .context("Failed to send cancel signal")?;

        self.authentication_required.store(false, Relaxed);
        Ok(())
    }
}

struct CustomAgent {
    authentication_required: Arc<AtomicBool>,
    passkey_receiver: Arc<Mutex<UnboundedReceiver<String>>>,
    cancel_signal_receiver: Arc<Mutex<UnboundedReceiver<()>>>,
}

impl Agent for CustomAgent {
    async fn request_passphrase(&self, _network: &Network) -> Result<String, Canceled> {
        let mut rx_key = self.passkey_receiver.lock().await;
        let mut rx_cancel = self.cancel_signal_receiver.lock().await;

        request_confirmation(
            self.authentication_required.clone(),
            &mut rx_key,
            &mut rx_cancel,
        )
        .await
        .map_err(|_| Canceled())
    }

    async fn request_private_key_passphrase(&self, _network: &Network) -> Result<String, Canceled> {
        let mut rx_key = self.passkey_receiver.lock().await;
        let mut rx_cancel = self.cancel_signal_receiver.lock().await;

        request_confirmation(
            self.authentication_required.clone(),
            &mut rx_key,
            &mut rx_cancel,
        )
        .await
        .map_err(|_| Canceled())
    }

    async fn request_user_name_and_passphrase(
        &self,
        _network: &Network,
    ) -> Result<(String, String), Canceled> {
        let mut rx_key = self.passkey_receiver.lock().await;
        let mut rx_cancel = self.cancel_signal_receiver.lock().await;

        let passphrase = request_confirmation(
            self.authentication_required.clone(),
            &mut rx_key,
            &mut rx_cancel,
        )
        .await
        .map_err(|_| Canceled())?;

        Ok((String::new(), passphrase))
    }

    async fn request_user_password(
        &self,
        _network: &Network,
        _user_name: Option<&String>,
    ) -> Result<String, Canceled> {
        let mut rx_key = self.passkey_receiver.lock().await;
        let mut rx_cancel = self.cancel_signal_receiver.lock().await;

        request_confirmation(
            self.authentication_required.clone(),
            &mut rx_key,
            &mut rx_cancel,
        )
        .await
        .map_err(|_| Canceled())
    }

    fn cancel(&self, _reason: CancellationReason) {
        self.authentication_required.store(false, Relaxed);
    }

    fn release(&self) {
        self.authentication_required.store(false, Relaxed);
    }
}

pub async fn request_confirmation(
    authentication_required: Arc<AtomicBool>,
    rx_key: &mut UnboundedReceiver<String>,
    rx_cancel: &mut UnboundedReceiver<()>,
) -> Result<String> {
    authentication_required.store(true, Relaxed);

    let result = tokio::select! {
        received_key = rx_key.recv() => {
            received_key.context("No key received")
        }
        received_cancel = rx_cancel.recv() => {
            received_cancel
                .context("Operation canceled by the user")
                .and(Err(anyhow!("Operation canceled")))
        }
    };

    authentication_required.store(false, Relaxed);
    result
}
