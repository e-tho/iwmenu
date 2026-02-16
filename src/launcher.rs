use anyhow::{anyhow, Context, Result};
use clap::ValueEnum;
use nix::{
    libc,
    sys::signal::{kill, killpg, Signal},
    unistd::Pid,
};
use process_wrap::std::{CommandWrap, ProcessGroup};
use signal_hook::iterator::Signals;
use std::{
    io::Write,
    process::{exit, Command, Stdio},
    sync::{
        atomic::{AtomicI32, Ordering},
        Once,
    },
    thread,
};

#[derive(Debug, Clone, ValueEnum)]
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
        program: String,
        args: Vec<String>,
    },
}

static CURRENT_LAUNCHER_PID: AtomicI32 = AtomicI32::new(-1);
static SIGNAL_HANDLER_INIT: Once = Once::new();

pub struct Launcher;

impl Launcher {
    pub fn run(cmd: LauncherCommand, input: Option<&str>) -> Result<Option<String>> {
        let command = match cmd {
            LauncherCommand::Fuzzel {
                icon_type,
                placeholder,
                password_mode,
            } => {
                let mut cmd = Command::new("fuzzel");
                cmd.arg("-d").arg("--minimal-lines");
                if icon_type == "font" {
                    cmd.arg("-I");
                }
                if let Some(hint_text) = placeholder {
                    cmd.arg("--placeholder").arg(hint_text);
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
                if let Some(hint_text) = placeholder {
                    cmd.arg("-theme-str")
                        .arg(format!("entry {{ placeholder: \"{hint_text}\"; }}"));
                }
                if password_mode {
                    cmd.arg("-password");
                }
                cmd
            }
            LauncherCommand::Dmenu { prompt } => {
                let mut cmd = Command::new("dmenu");
                if let Some(hint_text) = prompt {
                    cmd.arg("-p").arg(format!("{hint_text}: "));
                }
                cmd
            }
            LauncherCommand::Walker {
                placeholder,
                password_mode,
            } => {
                let mut cmd = Command::new("walker");
                cmd.arg("-d").arg("-k");
                if let Some(hint_text) = placeholder {
                    cmd.arg("-p").arg(hint_text);
                }
                if password_mode {
                    cmd.arg("-y");
                }
                cmd
            }
            LauncherCommand::Custom { program, args } => {
                let mut cmd = Command::new(&program);
                cmd.args(&args);
                cmd
            }
        };

        Self::run_command(command, input)
    }

    fn substitute_placeholders(
        template: &str,
        hint: Option<&str>,
        password_mode: bool,
    ) -> Result<String> {
        if !template.contains('{') {
            return Ok(template.to_string());
        }

        let mut result = template.to_string();

        if let Some(h) = hint {
            result = result.replace("{hint}", h);
            result = result.replace("{placeholder}", h);
            result = result.replace("{prompt}", &format!("{h}: "));
        } else {
            result = result.replace("{hint}", "");
            result = result.replace("{placeholder}", "");
            result = result.replace("{prompt}", "");
        }

        if result.contains("{password_flag:") {
            result = Self::replace_conditional_pattern(&result, "password_flag:", password_mode)?;
        }

        Ok(result)
    }

    fn replace_conditional_pattern(
        input: &str,
        pattern_prefix: &str,
        condition: bool,
    ) -> Result<String> {
        let start_pattern = format!("{{{pattern_prefix}");
        let mut result = String::with_capacity(input.len());
        let mut remaining = input;

        while let Some(start) = remaining.find(&start_pattern) {
            result.push_str(&remaining[..start]);

            let search_from = start + start_pattern.len();
            if let Some(end) = remaining[search_from..].find('}') {
                let abs_end = search_from + end;
                let value = &remaining[search_from..abs_end];

                if value.contains('{') || value.contains('}') {
                    return Err(anyhow!("Nested braces not supported in conditional patterns: {{{pattern_prefix}{value}}}"));
                }

                if condition {
                    result.push_str(value);
                }

                remaining = &remaining[abs_end + 1..];
            } else {
                return Err(anyhow!(
                    "Unclosed conditional pattern: {{{pattern_prefix}..."
                ));
            }
        }

        result.push_str(remaining);
        Ok(result)
    }

    fn parse_command(command_str: &str) -> Result<(String, Vec<String>)> {
        let parts =
            shlex::split(command_str).ok_or_else(|| anyhow!("Invalid shell syntax in command"))?;

        if parts.is_empty() {
            return Err(anyhow!("Empty command string"));
        }

        let program = parts[0].clone();
        let args = parts[1..].to_vec();

        Ok((program, args))
    }

    fn run_command(mut command: Command, input: Option<&str>) -> Result<Option<String>> {
        command.stdin(Stdio::piped()).stdout(Stdio::piped());

        let mut command_wrap = CommandWrap::from(command);
        command_wrap.wrap(ProcessGroup::leader());

        let mut child = command_wrap
            .spawn()
            .context("Failed to spawn launcher command")?;

        let pid = child.id() as i32;

        SIGNAL_HANDLER_INIT.call_once(|| {
            thread::spawn(|| {
                let mut signals = Signals::new([libc::SIGTERM, libc::SIGINT]).unwrap();
                if let Some(_signal) = signals.forever().next() {
                    let current_pid = CURRENT_LAUNCHER_PID.load(Ordering::Relaxed);
                    if current_pid > 0 && kill(Pid::from_raw(current_pid), None).is_ok() {
                        let _ = killpg(Pid::from_raw(current_pid), Signal::SIGTERM);
                    }
                    exit(0);
                }
            });
        });

        CURRENT_LAUNCHER_PID.store(pid, Ordering::Relaxed);

        if let Some(input_data) = input {
            if let Some(stdin) = child.stdin().as_mut() {
                stdin.write_all(input_data.as_bytes())?;
            }
        }

        let output = child.wait_with_output()?;
        let trimmed_output = String::from_utf8_lossy(&output.stdout).trim().to_string();

        CURRENT_LAUNCHER_PID.store(-1, Ordering::Relaxed);

        if trimmed_output.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed_output))
        }
    }

    pub fn create_command(
        launcher_type: &LauncherType,
        command_str: &Option<String>,
        icon_type: &str,
        hint: Option<&str>,
        password_mode: bool,
    ) -> Result<LauncherCommand> {
        let hint_text = hint.filter(|h| !h.is_empty()).map(|h| h.to_string());

        match launcher_type {
            LauncherType::Fuzzel => Ok(LauncherCommand::Fuzzel {
                icon_type: icon_type.to_string(),
                placeholder: hint_text,
                password_mode,
            }),
            LauncherType::Rofi => Ok(LauncherCommand::Rofi {
                icon_type: icon_type.to_string(),
                placeholder: hint_text,
                password_mode,
            }),
            LauncherType::Dmenu => Ok(LauncherCommand::Dmenu { prompt: hint_text }),
            LauncherType::Walker => Ok(LauncherCommand::Walker {
                placeholder: hint_text,
                password_mode,
            }),
            LauncherType::Custom => {
                if let Some(cmd) = command_str {
                    let processed_cmd = Self::substitute_placeholders(cmd, hint, password_mode)?;
                    let (program, args) = Self::parse_command(&processed_cmd)?;

                    Ok(LauncherCommand::Custom { program, args })
                } else {
                    Err(anyhow!("No custom launcher command provided"))
                }
            }
        }
    }
}
