use anyhow::Result;
use clap::{builder::EnumValueParser, Arg, Command};
use iwmenu::{app::App, menu::Menu};
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

    tokio::spawn(async move {
        while let Some(log) = log_receiver.recv().await {
            println!("LOG: {}", log);
        }
    });

    let app = App::new(menu.clone(), log_sender).await?;
    app.run(menu).await?;

    Ok(())
}
