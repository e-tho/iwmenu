use anyhow::{anyhow, Context, Result};
use clap::ArgEnum;
use nix::sys::signal::{killpg, Signal};
use nix::unistd::Pid;
use process_wrap::std::{ProcessGroup, StdCommandWrap};
use regex::Regex;
use shlex::Shlex;
use signal_hook::iterator::Signals;
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;

#[derive(Debug, Clone, ArgEnum)]
pub enum LauncherType {
    Fuzzel,
    Rofi,
    Dmenu,
    Walker,
    Custom,
}

#[derive(Debug, Clone)]
pub enum LauncherCommand {
    Fuzzel {
        icon_type: String,
        placeholder: Option<String>,
        password_mode: bool,
    },
    Rofi {
        icon_type: String,
        placeholder: Option<String>,
        password_mode: bool,
    },
    Dmenu {
        prompt: Option<String>,
    },
    Walker {
        placeholder: Option<String>,
        password_mode: bool,
    },
    Custom {
        command: String,
        args: Vec<(String, String)>,
    },
}

pub struct Launcher;

impl Launcher {
    pub fn run(menu_cmd: LauncherCommand, input: Option<&str>) -> Result<Option<String>> {
        let command = match menu_cmd {
            LauncherCommand::Fuzzel {
                icon_type,
                placeholder,
                password_mode,
            } => {
                let mut cmd = Command::new("fuzzel");
                cmd.arg("-d");
                if icon_type == "font" {
                    cmd.arg("-I");
                }
                if let Some(placeholder_text) = placeholder {
                    cmd.arg("--placeholder").arg(placeholder_text);
                }
                if password_mode {
                    cmd.arg("--password");
                }
                cmd
            }
            LauncherCommand::Rofi {
                icon_type,
                placeholder,
                password_mode,
            } => {
                let mut cmd = Command::new("rofi");
                cmd.arg("-m").arg("-1").arg("-dmenu");
                if icon_type == "xdg" {
                    cmd.arg("-show-icons");
                }
                if let Some(placeholder_text) = placeholder {
                    cmd.arg("-theme-str").arg(format!(
                        "entry {{ placeholder: \"{}\"; }}",
                        placeholder_text
                    ));
                }
                if password_mode {
                    cmd.arg("-password");
                }
                cmd
            }
            LauncherCommand::Dmenu { prompt } => {
                let mut cmd = Command::new("dmenu");
                if let Some(prompt_text) = prompt {
                    cmd.arg("-p").arg(format!("{}: ", prompt_text));
                }
                cmd
            }
            LauncherCommand::Walker {
                placeholder,
                password_mode,
            } => {
                let mut cmd = Command::new("walker");
                cmd.arg("-d").arg("-k");
                if let Some(placeholder_text) = placeholder {
                    cmd.arg("-p").arg(placeholder_text);
                }
                if password_mode {
                    cmd.arg("-y");
                }
                cmd
            }
            LauncherCommand::Custom { command, args } => {
                let mut cmd_str = command;

                for (key, value) in args {
                    cmd_str = cmd_str.replace(&format!("{{{}}}", key), &value);
                }

                let re = Regex::new(r"\{(\w+):([^\}]+)\}").unwrap();
                cmd_str = re
                    .replace_all(&cmd_str, |caps: &regex::Captures| {
                        let placeholder_name = &caps[1];
                        let default_value = &caps[2];
                        match placeholder_name {
                            "password_flag" => default_value.to_string(),
                            _ => caps[0].to_string(),
                        }
                    })
                    .to_string();

                cmd_str = cmd_str.replace("{placeholder}", "");

                let parts: Vec<String> = Shlex::new(&cmd_str).collect();
                let (cmd_program, args) = parts
                    .split_first()
                    .ok_or_else(|| anyhow!("Failed to parse custom menu command"))?;

                let mut cmd = Command::new(cmd_program);
                cmd.args(args);
                cmd
            }
        };

        Self::run_command(command, input)
    }

    fn run_command(mut command: Command, input: Option<&str>) -> Result<Option<String>> {
        command.stdin(Stdio::piped()).stdout(Stdio::piped());

        let mut command_wrap = StdCommandWrap::from(command);
        command_wrap.wrap(ProcessGroup::leader());

        let mut child = command_wrap
            .spawn()
            .context("Failed to spawn menu command")?;

        let pid = child.id() as i32;
        thread::spawn(move || {
            let mut signals = Signals::new([libc::SIGTERM, libc::SIGINT]).unwrap();
            for _signal in signals.forever() {
                let _ = killpg(Pid::from_raw(pid), Signal::SIGTERM);
            }
        });

        if let Some(input_data) = input {
            if let Some(stdin) = child.stdin().as_mut() {
                stdin.write_all(input_data.as_bytes())?;
            }
        }

        let output = child.wait_with_output()?;
        let trimmed_output = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if trimmed_output.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed_output))
        }
    }

    pub fn create_command(
        menu_type: &LauncherType,
        command_str: &Option<String>,
        icon_type: &str,
        prompt: Option<&str>,
        placeholder: Option<&str>,
        password_mode: bool,
    ) -> Result<LauncherCommand> {
        let placeholder_text = placeholder.map(|p| p.to_string());

        match menu_type {
            LauncherType::Fuzzel => Ok(LauncherCommand::Fuzzel {
                icon_type: icon_type.to_string(),
                placeholder: placeholder_text,
                password_mode,
            }),
            LauncherType::Rofi => Ok(LauncherCommand::Rofi {
                icon_type: icon_type.to_string(),
                placeholder: placeholder_text,
                password_mode,
            }),
            LauncherType::Dmenu => Ok(LauncherCommand::Dmenu {
                prompt: prompt.map(|p| p.to_string()),
            }),
            LauncherType::Walker => Ok(LauncherCommand::Walker {
                placeholder: placeholder_text,
                password_mode,
            }),
            LauncherType::Custom => {
                if let Some(cmd) = command_str {
                    let mut args = Vec::new();

                    if let Some(p) = prompt {
                        args.push(("prompt".to_string(), p.to_string()));
                    }

                    if let Some(p) = placeholder {
                        args.push(("placeholder".to_string(), p.to_string()));
                    }

                    if password_mode {
                        args.push(("password_flag".to_string(), "--password".to_string()));
                    }

                    Ok(LauncherCommand::Custom {
                        command: cmd.clone(),
                        args,
                    })
                } else {
                    Err(anyhow!("No custom menu command provided"))
                }
            }
        }
    }
}
