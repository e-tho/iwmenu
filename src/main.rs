use anyhow::Result;
use clap::{builder::EnumValueParser, Arg, Command};
use iwmenu::{app::App, menu::Menu};
use notify_rust::{Notification, Timeout};
use tokio::sync::mpsc::unbounded_channel;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("dmenu")
                .short('d')
                .long("dmenu")
                .takes_value(true)
                .required(true)
                .value_parser(EnumValueParser::<Menu>::new())
                .default_value("dmenu")
                .help("Dmenu backend to use (dmenu, rofi, wofi, fuzzel)"),
        )
        .get_matches();

    let menu: Menu = matches.get_one::<Menu>("dmenu").cloned().unwrap();

    let (log_sender, mut log_receiver) = unbounded_channel::<String>();
    let (notification_sender, mut notification_receiver) = unbounded_channel::<(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<Timeout>,
    )>();

    tokio::spawn(async move {
        while let Some(log) = log_receiver.recv().await {
            println!("LOG: {}", log);
        }
    });

    tokio::spawn(async move {
        while let Some((summary, body, icon, timeout)) = notification_receiver.recv().await {
            let mut notification = Notification::new();

            let summary_str = summary.as_deref().unwrap_or("iNet Wireless");
            notification.summary(summary_str);

            if let Some(ref body) = body {
                notification.body(body);
            }

            let icon_str = icon.as_deref().unwrap_or("network-wireless");
            notification.icon(icon_str);

            notification.timeout(timeout.unwrap_or(Timeout::Milliseconds(3000)));

            if let Err(err) = notification.show() {
                eprintln!("Failed to send notification: {}", err);
            }
        }
    });

    let mut app: App = App::new(
        menu.clone(),
        log_sender.clone(),
        notification_sender.clone(),
    )
    .await?;

    if let Some(ssid) = app.run(menu).await? {
        log_sender
            .send(format!("Connected to network: {}", ssid))
            .unwrap_or_else(|err| println!("Failed to send message: {}", err));
    }

    Ok(())
}
