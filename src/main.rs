use tokio::runtime::Runtime;
use tokio::sync::watch;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod can;
mod gui;
mod miu_state;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env_lossy();
    let fmt = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt)
        .init();

    let runtime = Runtime::new().expect("Unable to create tokio runtime");

    // Keeping this guard around is needed for `tokio::spawn` to work.
    let _guard = runtime.enter();

    let (interfaces_client, interfaces_task) = can::interfaces::task();
    let (can_client, mut can_task) = can::task(runtime.handle().clone());

    // We don't need the receiver right now and we can just create more receivers from the sender,
    // so we can just drop it here.
    let (miu_state_sender, _) = watch::channel(miu_state::MiuState::default());

    std::thread::spawn(move || {
        runtime.block_on(async {
            // `join!` runs all futures on the same thread. By spawning new tasks and passing the
            // join handles to `join!` we allow the tasks to run in parallel. For this application
            // it's not that important because performance is not an issue, but good to know
            // anyway.
            let interfaces_handle = tokio::spawn(async move { interfaces_task.run().await });
            let can_handle = tokio::spawn(async move { can_task.run().await });

            // When this join returns it means the background tasks have died, which is bad. This
            // situation will be detected when trying to communicate with the tasks though, so we
            // don't have to handle this situation here.
            let _ = tokio::join!(interfaces_handle, can_handle);
        })
    });

    let gui = Box::new(gui::Gui {
        can: can_client,
        interfaces: interfaces_client,
        selected_interface: None,
        miu_state: Default::default(),
        miu_state_sender,
    });

    eframe::run_native(
        "miu",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            gui
        }),
    )
    .expect("Failed to run GUI");

    Ok(())
}
