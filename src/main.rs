use tokio::runtime::Runtime;
use tokio::sync::watch;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod can;
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

    let app = Box::new(App {
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
            app
        }),
    )
    .expect("Failed to run GUI");

    Ok(())
}

type MiuStateSender = watch::Sender<miu_state::MiuState>;
type MiuStateReceiver = watch::Receiver<miu_state::MiuState>;

struct App {
    can: can::CanClient,
    interfaces: can::interfaces::InterfacesClient,
    selected_interface: Option<String>,
    miu_state: miu_state::MiuState,
    miu_state_sender: MiuStateSender,
}

impl eframe::App for App {
    fn update(&mut self, context: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top-bar").show(context, |ui| {
            self.top_bar(ui);
        });

        egui::CentralPanel::default().show(context, |ui| {
            self.control_grid(ui);
        });

        // This operation fails if there are no receivers, which is the case when there is no
        // active can bus connection. For the render loop of the gui this is not important so we
        // can just ignore this case.
        let _ = self.miu_state_sender.send(self.miu_state);
    }
}

impl App {
    fn top_bar(&mut self, ui: &mut eframe::egui::Ui) {
        ui.horizontal(|ui| {
            egui::widgets::global_dark_light_mode_switch(ui);

            ui.separator();

            let interfaces = self
                .interfaces
                .get()
                .expect("Failed to get available can interfaces");

            let connection_state = self
                .can
                .state()
                .expect("Failed to get can connection state");

            ui.add_enabled_ui(connection_state == can::State::Disconnected, |ui| {
                ui.label("interface");

                egui::ComboBox::from_id_source("can-interface-selector")
                    .selected_text(match self.selected_interface.clone() {
                        Some(interface) => interface,
                        None => String::from("None"),
                    })
                    .show_ui(ui, |ui| {
                        for interface in interfaces {
                            ui.selectable_value(
                                &mut self.selected_interface,
                                Some(interface.clone()),
                                interface,
                            );
                        }
                    });
            });

            if let Some(interface) = &self.selected_interface {
                match connection_state {
                    can::State::Connected => {
                        if ui.button("Disconnect").clicked() {
                            self.can.disconnect().expect("Failed to disconnect");
                        }
                    }
                    can::State::Disconnected => {
                        if ui.button("Connect").clicked() {
                            self.can
                                .connect(interface.clone(), self.miu_state_sender.subscribe())
                                .expect("Failed to connect");
                        }
                    }
                }
            }
        });
    }

    fn control_grid(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("control_signal_grid")
            .num_columns(3)
            .spacing([40.0, 10.0])
            .striped(true)
            .show(ui, |ui| {
                ui.heading("Engine speed");
                ui.add(
                    egui::DragValue::new(&mut self.miu_state.engine_speed)
                        .speed(25)
                        .clamp_range(0_u16..=7000)
                        .suffix(" rpm"),
                );

                ui.checkbox(&mut self.miu_state.engine_speed_fault, "Fault");
                ui.end_row();

                ui.heading("Boost");
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut self.miu_state.boost, 0..=255).show_value(false));
                    ui.label(format!(
                        "{:.0}%",
                        self.miu_state.get_boost_percentage() * 100.0
                    ));
                });
                ui.end_row();

                ui.heading("Coolant temperature 1");
                ui.add(
                    egui::DragValue::new(&mut self.miu_state.coolant_temperature)
                        .clamp_range(0_u16..=150)
                        .suffix("°C"),
                );
                ui.checkbox(&mut self.miu_state.coolant_temperature_fault, "Fault");
                ui.end_row();

                ui.heading("Check engine");
                ui.checkbox(&mut self.miu_state.check_engine, "");
                ui.end_row();

                ui.heading("Cruise");
                ui.checkbox(&mut self.miu_state.cruise, "");
                ui.end_row();

                ui.heading("Gear lever");
                egui::ComboBox::from_id_source("gear-lever")
                    .selected_text(format!(
                        "{:?}",
                        can::tcm::Gear::from(self.miu_state.gear_lever)
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.miu_state.gear_lever,
                            can::tcm::Gear::Unknown.into(),
                            format!("{:?}", can::tcm::Gear::Unknown),
                        );
                        ui.selectable_value(
                            &mut self.miu_state.gear_lever,
                            can::tcm::Gear::Park.into(),
                            format!("{:?}", can::tcm::Gear::Park),
                        );
                        ui.selectable_value(
                            &mut self.miu_state.gear_lever,
                            can::tcm::Gear::Reverse.into(),
                            format!("{:?}", can::tcm::Gear::Reverse),
                        );
                        ui.selectable_value(
                            &mut self.miu_state.gear_lever,
                            can::tcm::Gear::Neutral.into(),
                            format!("{:?}", can::tcm::Gear::Neutral),
                        );
                        ui.selectable_value(
                            &mut self.miu_state.gear_lever,
                            can::tcm::Gear::Drive.into(),
                            format!("{:?}", can::tcm::Gear::Drive),
                        );
                        ui.selectable_value(
                            &mut self.miu_state.gear_lever,
                            can::tcm::Gear::Limit3.into(),
                            format!("{:?}", can::tcm::Gear::Limit3),
                        );
                        ui.selectable_value(
                            &mut self.miu_state.gear_lever,
                            can::tcm::Gear::Limit2.into(),
                            format!("{:?}", can::tcm::Gear::Limit2),
                        );
                        ui.selectable_value(
                            &mut self.miu_state.gear_lever,
                            can::tcm::Gear::Limit1.into(),
                            format!("{:?}", can::tcm::Gear::Limit1),
                        );
                    });
                ui.checkbox(&mut self.miu_state.gear_lever_fault, "Fault");
                ui.end_row();

                ui.heading("Actual gear");
                egui::ComboBox::from_id_source("actual-gear")
                    .selected_text(format!(
                        "{:?}",
                        can::tcm::Gear::from(self.miu_state.actual_gear)
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.miu_state.actual_gear,
                            can::tcm::Gear::Unknown.into(),
                            format!("{:?}", can::tcm::Gear::Unknown),
                        );
                        ui.selectable_value(
                            &mut self.miu_state.actual_gear,
                            can::tcm::Gear::Park.into(),
                            format!("{:?}", can::tcm::Gear::Park),
                        );
                        ui.selectable_value(
                            &mut self.miu_state.actual_gear,
                            can::tcm::Gear::Reverse.into(),
                            format!("{:?}", can::tcm::Gear::Reverse),
                        );
                        ui.selectable_value(
                            &mut self.miu_state.actual_gear,
                            can::tcm::Gear::Neutral.into(),
                            format!("{:?}", can::tcm::Gear::Neutral),
                        );
                        ui.selectable_value(
                            &mut self.miu_state.actual_gear,
                            can::tcm::Gear::Drive.into(),
                            format!("{:?}", can::tcm::Gear::Drive),
                        );
                        ui.selectable_value(
                            &mut self.miu_state.actual_gear,
                            can::tcm::Gear::Limit3.into(),
                            format!("{:?}", can::tcm::Gear::Limit3),
                        );
                        ui.selectable_value(
                            &mut self.miu_state.actual_gear,
                            can::tcm::Gear::Limit2.into(),
                            format!("{:?}", can::tcm::Gear::Limit2),
                        );
                        ui.selectable_value(
                            &mut self.miu_state.actual_gear,
                            can::tcm::Gear::Limit1.into(),
                            format!("{:?}", can::tcm::Gear::Limit1),
                        );
                    });
                ui.checkbox(&mut self.miu_state.actual_gear_fault, "Fault");
                ui.end_row();

                ui.heading("Sport");
                ui.checkbox(&mut self.miu_state.sport, "");
                ui.end_row();

                ui.heading("Winter");
                ui.checkbox(&mut self.miu_state.winter, "");
                ui.end_row();

                ui.heading("Check gearbox");
                ui.checkbox(&mut self.miu_state.check_gearbox, "");
                ui.end_row();
            });
    }
}
