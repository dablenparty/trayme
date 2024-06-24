use anyhow::Context;
use clap::{Parser, ValueHint};
use strum::VariantArray;
use tao::event_loop::EventLoopBuilder;
use tray_icon::{
    menu::{Menu, MenuItemBuilder},
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
    cmd: String,
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

fn build_tray_menu() -> anyhow::Result<Menu> {
    let menu = Menu::new();
    for msg in TrayMessage::VARIANTS {
        let item = MenuItemBuilder::new()
            .text(msg.to_string())
            .id(msg.into())
            .build();
        menu.append(&item)?;
    }
    Ok(menu)
}

fn build_tray() -> anyhow::Result<TrayIcon> {
    let menu = build_tray_menu()?;
    TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("PROCESS_NAME")
        .build()
        .context("Failed to build tray icon")
}

fn run_event_loop() -> anyhow::Result<()> {
    // TODO: show notification that the app is running
    let event_loop = EventLoopBuilder::new().build();

    // TODO: build tray here
    // tray must be built AFTER event loop to prevent initializing low-level
    // libraries out of order (mostly a macOS issue)

    // TODO: log all output from the wrapped command

    event_loop.run(move |_event, _window, control_flow| {
        *control_flow = tao::event_loop::ControlFlow::Exit;
    })
}

fn main() {
    let args = CliArgs::parse();
    #[cfg(debug_assertions)]
    println!("{args:#?}");
    run_event_loop().unwrap();
}
