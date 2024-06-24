use tao::event_loop::EventLoopBuilder;

fn run_event_loop() -> anyhow::Result<()> {
    // TODO: show notification that the app is running
    let event_loop = EventLoopBuilder::new().build();

    // TODO: build tray here
    // tray must be built AFTER event loop to prevent initializing low-level
    // libraries out of order (mostly a macOS issue)

    // TODO: log all output from the wrapped command

    event_loop.run(|_event, _window, control_flow| {
        *control_flow = tao::event_loop::ControlFlow::Exit;
    })
}

fn main() {
    println!("Hello, world!");
    run_event_loop().unwrap();
}
