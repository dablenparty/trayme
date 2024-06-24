use anyhow::Context;
use strum::VariantArray;
use tao::event_loop::EventLoopBuilder;
use tray_icon::{
    menu::{Menu, MenuItemBuilder},
    TrayIcon, TrayIconBuilder,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::VariantArray)]
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
    println!("Hello, world!");
    run_event_loop().unwrap();
}
