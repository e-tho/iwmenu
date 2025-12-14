use anyhow::{anyhow, Result};
use clap::{builder::EnumValueParser, Arg, Command};
use iwmenu::{app::App, icons::Icons, launcher::LauncherType, menu::Menu};
use rust_i18n::{i18n, set_locale};
use std::{env, sync::Arc};
use sys_locale::get_locale;

i18n!("locales", fallback = "en");

fn validate_launcher_command(command: &str) -> Result<String, String> {
    if command.contains("{placeholder}") {
        eprintln!("WARNING: {{placeholder}} is deprecated. Use {{hint}} instead.");
    }
    if command.contains("{prompt}") {
        eprintln!("WARNING: {{prompt}} is deprecated. Use {{hint}} instead.");
    }

    Ok(command.to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let locale = get_locale().unwrap_or_else(|| {
        eprintln!("Locale not detected, defaulting to 'en'.");
        String::from("en")
    });
    set_locale(&locale);

    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("launcher")
                .short('l')
                .long("launcher")
                .required(true)
                .takes_value(true)
                .value_parser(EnumValueParser::<LauncherType>::new())
                .conflicts_with("menu")
                .help("Launcher to use (replaces deprecated --menu)"),
        )
        .arg(
            Arg::new("menu") // deprecated
                .short('m')
                .long("menu")
                .takes_value(true)
                .value_parser(EnumValueParser::<LauncherType>::new())
                .hide(true)
                .help("DEPRECATED: use --launcher instead"),
        )
        .arg(
            Arg::new("launcher_command")
                .long("launcher-command")
                .takes_value(true)
                .required_if_eq("launcher", "custom")
                .conflicts_with("menu_command")
                .value_parser(validate_launcher_command)
                .help("Launcher command to use when --launcher is set to custom"),
        )
        .arg(
            Arg::new("menu_command") // deprecated
                .long("menu-command")
                .takes_value(true)
                .required_if_eq("menu", "custom")
                .hide(true)
                .value_parser(validate_launcher_command)
                .help("DEPRECATED: use --launcher-command instead"),
        )
        .arg(
            Arg::new("icon")
                .short('i')
                .long("icon")
                .takes_value(true)
                .possible_values(["font", "xdg"])
                .default_value("font")
                .help("Choose the type of icons to use"),
        )
        .arg(
            Arg::new("spaces")
                .short('s')
                .long("spaces")
                .takes_value(true)
                .default_value("1")
                .help("Number of spaces between icon and text when using font icons"),
        )
        .get_matches();

    let launcher_type: LauncherType = if matches.contains_id("launcher") {
        matches
            .get_one::<LauncherType>("launcher")
            .cloned()
            .unwrap()
    } else if matches.contains_id("menu") {
        eprintln!("WARNING: --menu flag is deprecated. Please use --launcher instead.");
        matches.get_one::<LauncherType>("menu").cloned().unwrap()
    } else {
        LauncherType::Dmenu
    };

    let command_str = if matches.contains_id("launcher_command") {
        matches.get_one::<String>("launcher_command").cloned()
    } else if matches.contains_id("menu_command") {
        eprintln!(
            "WARNING: --menu-command flag is deprecated. Please use --launcher-command instead."
        );
        matches.get_one::<String>("menu_command").cloned()
    } else {
        None
    };

    let icon_type = matches.get_one::<String>("icon").cloned().unwrap();

    let icons = Arc::new(Icons::new());
    let menu = Menu::new(launcher_type, icons.clone());

    let spaces = matches
        .get_one::<String>("spaces")
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or_else(|| anyhow!("Invalid value for --spaces. Must be a positive integer."))?;

    run_app_loop(&menu, &command_str, &icon_type, spaces, icons).await?;

    Ok(())
}

async fn run_app_loop(
    menu: &Menu,
    command_str: &Option<String>,
    icon_type: &str,
    spaces: usize,
    icons: Arc<Icons>,
) -> Result<()> {
    let mut app = App::new(icons.clone()).await?;

    loop {
        match app.run(menu, command_str, icon_type, spaces).await {
            Ok(_) => {
                if !app.reset_mode {
                    break;
                }
            }
            Err(err) => {
                eprintln!("Error during app execution: {err:?}");

                if !app.reset_mode {
                    return Err(anyhow!("Fatal error in application: {err}"));
                }
            }
        }

        if app.reset_mode {
            app = App::new(icons.clone()).await?;
            app.reset_mode = false;
        }
    }

    Ok(())
}
