use crate::can;
use crate::miu_state;
use tokio::sync::watch;

pub struct Gui {
    pub can: can::CanClient,
    pub interfaces: can::interfaces::InterfacesClient,
    pub selected_interface: Option<String>,
    pub miu_state: miu_state::MiuState,
    pub miu_state_sender: watch::Sender<miu_state::MiuState>,
}

impl eframe::App for Gui {
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

impl Gui {
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
