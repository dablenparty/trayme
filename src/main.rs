use std::{fs::OpenOptions, path::PathBuf, process, str::FromStr};

use anyhow::Context;
use clap::{Parser, ValueHint};
use env_logger::Target;
use log::{debug, error, info};
use strum::VariantArray;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuEventReceiver, MenuItemBuilder},
    TrayIcon, TrayIconBuilder,
};

/// Runs any command-line command in the system tray. This is meant for long-running
/// background processes that the user wants to keep running without having to keep a
/// terminal window open, but it'll work with any command.
#[derive(Debug, Parser)]
#[command(trailing_var_arg = true, about, version, author)]
struct CliArgs {
    /// The command to run.
    #[arg(required = true, value_hint = ValueHint::CommandWithArguments, num_args = 1..)]
    cmd: Vec<String>,
    // TODO: customize tray icon via cli (e.g. tooltip, icon, etc.)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantArray)]
enum TrayMessage {
    Kill,
    ShowLogs,
}

impl std::fmt::Display for TrayMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrayMessage::Kill => write!(f, "Kill"),
            TrayMessage::ShowLogs => write!(f, "Show Logs"),
        }
    }
}

impl FromStr for TrayMessage {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Kill" => Ok(TrayMessage::Kill),
            "Show Logs" => Ok(TrayMessage::ShowLogs),
            _ => Err(strum::ParseError::VariantNotFound),
        }
    }
}

fn build_tray_menu() -> anyhow::Result<Menu> {
    let menu = Menu::new();
    for msg in TrayMessage::VARIANTS {
        let item = MenuItemBuilder::new()
            .text(msg.to_string())
            .id(msg.into())
            .enabled(true)
            .build();
        menu.append(&item)?;
    }
    Ok(menu)
}

fn build_tray(tooltip: impl AsRef<str>) -> anyhow::Result<TrayIcon> {
    // TODO: tray icon
    let menu = build_tray_menu()?;
    TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(tooltip)
        .build()
        .context("Failed to build tray icon")
}

/// Handles tray events in the event loop. Returns a [`tao::event_loop::ControlFlow`]
/// to be used by the next iteration of the event loop.
fn run_event_loop(
    child_proc: &mut process::Child,
    menu_channel: &MenuEventReceiver,
) -> anyhow::Result<ControlFlow> {
    if let Some(status) = child_proc.try_wait()? {
        if !status.success() {
            error!("Command exited with status: {status:#}");
        } else {
            info!("Command exited successfully: {status:#}");
        }
        return Ok(ControlFlow::Exit);
    }

    if let Ok(event) = menu_channel.try_recv() {
        debug!("{event:?}");

        let msg = TrayMessage::from_str(&event.id().0)?;

        match msg {
            TrayMessage::Kill => {
                child_proc.kill().context("Failed to kill child process")?;
                return Ok(ControlFlow::Exit);
            }
            TrayMessage::ShowLogs => {
                let logs_dir = get_logs_dir()?;
                open::that(logs_dir).context("Failed to open logs dir")?;
            }
        }
    }

    Ok(ControlFlow::Poll)
}

fn get_logs_dir() -> anyhow::Result<PathBuf> {
    let mut logs_dir = dirs::data_dir().context("Failed to get data directory")?;
    logs_dir.push(env!("CARGO_PKG_NAME"));
    std::fs::create_dir_all(&logs_dir).context("Failed to create logs directory")?;
    Ok(logs_dir)
}

fn init_logging() -> anyhow::Result<()> {
    let target = if cfg!(debug_assertions) {
        Target::Stdout
    } else {
        let log_dir = get_logs_dir()?;
        let log_file = log_dir.join("log.log");
        let writer = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .context("Failed to open log file")?;
        Target::Pipe(Box::new(writer))
    };

    env_logger::Builder::from_default_env()
        .target(target)
        .init();

    Ok(())
}

fn main() -> anyhow::Result<()> {
    init_logging()?;
    let args = CliArgs::parse();
    debug!("{args:#?}");
    // TODO: notifications

    // TODO: show notification that the app is running
    let event_loop = EventLoopBuilder::new().build();

    let CliArgs { cmd } = args;
    // tray must be built AFTER event loop to prevent initializing low-level
    // libraries out of order (mostly a macOS issue)
    let mut tray = Some(build_tray(cmd.join(" "))?);
    let menu_channel = MenuEvent::receiver();

    // TODO: log all output from the wrapped command
    let mut child_proc = process::Command::new(&cmd[0])
        .args(&cmd[1..])
        .spawn()
        .context("Failed to spawn command")?;

    event_loop.run(move |_event, _window, control_flow| {
        *control_flow = tao::event_loop::ControlFlow::Poll;

        match run_event_loop(&mut child_proc, menu_channel) {
            Ok(cf) => *control_flow = cf,
            Err(err) => {
                error!("Error: {err:#}");
                *control_flow = ControlFlow::Exit;
                let _ = tray.take();
            }
        };
    })
}
