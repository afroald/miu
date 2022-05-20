// use deku::prelude::*;
use eframe::egui;
use std::mem::drop;
use std::sync::{Arc, Mutex};

mod can;

pub struct MiuState {
    engine_speed: u16,
    engine_speed_fault: bool,
    vehicle_speed: u16,
    vehicle_speed_fault: bool,
    boost: u8,
    coolant_temperature_1: u8,
    coolant_temperature_1_fault: bool,
    coolant_temperature_2: u8,
    coolant_temperature_2_fault: bool,
    fuel_level: u16,
    fuel_level_fault: bool,
}

impl Default for MiuState {
    fn default() -> Self {
        MiuState {
            engine_speed: 0,
            engine_speed_fault: false,
            vehicle_speed: 0,
            vehicle_speed_fault: false,
            boost: 0,
            coolant_temperature_1: 0,
            coolant_temperature_1_fault: false,
            coolant_temperature_2: 0,
            coolant_temperature_2_fault: false,
            fuel_level: 0,
            fuel_level_fault: false,
        }
    }
}

impl MiuState {
    fn get_boost_percentage(&self) -> f32 {
        f32::from(self.boost) / 255.0
    }

    fn set_boost_percentage(&mut self, percentage: f32) {
        if percentage > 100.0 {
            panic!("Received percentage above 100%: {}", percentage);
        }

        self.boost = (255.0 * percentage).floor() as u8;
    }
}

#[derive(Debug, PartialEq)]
enum Mode {
    Display,
    Control,
}

struct MiuComApp {
    mode: Mode,
    can_interfaces: can::interfaces::CanInterfaces,
    selected_can_interface: Option<String>,
    reader: can::reader::Reader,
    state: Arc<Mutex<MiuState>>,
}

impl Default for MiuComApp {
    fn default() -> Self {
        Self {
            mode: Mode::Display,
            can_interfaces: can::interfaces::CanInterfaces::new(),
            selected_can_interface: None,
            reader: can::reader::Reader::new(),
            state: Arc::new(Mutex::new(MiuState::default())),
        }
    }
}

impl eframe::App for MiuComApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top-bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::widgets::global_dark_light_mode_switch(ui);

                ui.separator();

                ui.label("Mode:");
                egui::ComboBox::from_id_source("mode-selector")
                    .selected_text(format!("{:?}", self.mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.mode, Mode::Display, "Display");
                        ui.selectable_value(&mut self.mode, Mode::Control, "Control");
                    });

                ui.separator();

                let interfaces = self.can_interfaces.lock().unwrap();
                ui.label("Bus:");

                match &*interfaces {
                    Ok(interfaces) => {
                        egui::ComboBox::from_id_source("can-interface-selector")
                            .selected_text(match self.selected_can_interface.clone() {
                                Some(interface) => interface,
                                None => String::from("None"),
                            })
                            .show_ui(ui, |ui| {
                                for interface in interfaces {
                                    ui.selectable_value(
                                        &mut self.selected_can_interface,
                                        Some(interface.clone()),
                                        interface,
                                    );
                                }
                            });

                        let connection_state = self.reader.connection_state.lock().unwrap();
                        let connection_state_copy = connection_state.clone();
                        drop(connection_state);
                        match connection_state_copy {
                            can::reader::ConnectionState::Connected => {
                                if ui.button("Disconnect").clicked() {
                                    self.reader.disconnect();
                                }
                            }
                            can::reader::ConnectionState::Disconnected => {
                                if let Some(interface) = &self.selected_can_interface {
                                    if ui.button("Connect").clicked() {
                                        self.reader.connect(&interface, Arc::clone(&self.state));
                                    }
                                }
                            }
                        }
                    }
                    Err(error) => {
                        ui.label(format!("Error getting can interfaces: {}", error));
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.mode {
            Mode::Display => self.display_grid_contents(ui),
            Mode::Control => self.control_grid_contents(ui),
        });

        ctx.request_repaint();
    }
}

impl MiuComApp {
    fn display_grid_contents(&self, ui: &mut egui::Ui) {
        let state = self.state.lock().unwrap();

        egui::Grid::new("signal_grid")
            .num_columns(3)
            .spacing([40.0, 10.0])
            .striped(true)
            .show(ui, |ui| {
                ui.heading("Engine speed");
                ui.heading(format!("{} rpm", state.engine_speed));
                ui.label(format!("Fault: {}", state.engine_speed_fault));
                ui.end_row();

                ui.heading("Vehicle speed");
                ui.heading(format!("{} km/h", state.vehicle_speed));
                ui.label(format!("Fault: {}", state.vehicle_speed_fault));
                ui.end_row();

                ui.heading("Boost");
                ui.add(egui::ProgressBar::new(state.get_boost_percentage()).show_percentage());
                ui.end_row();

                ui.heading("Coolant temperature 1");
                ui.heading(format!("{}°C", state.coolant_temperature_1));
                ui.label(format!("Fault: {}", state.coolant_temperature_1_fault));
                ui.end_row();

                ui.heading("Coolant temperature 2");
                ui.heading(format!("{}°C", state.coolant_temperature_2));
                ui.label(format!("Fault: {}", state.coolant_temperature_2_fault));
                ui.end_row();

                ui.heading("Fuel level");
                ui.heading(format!("{} cc", state.fuel_level));
                ui.label(format!("Fault: {}", state.fuel_level_fault));
                ui.end_row();
            });
    }

    fn control_grid_contents(&mut self, ui: &mut egui::Ui) {
        let mut state = self.state.lock().unwrap();

        egui::Grid::new("control_signal_grid")
            .num_columns(3)
            .spacing([40.0, 10.0])
            .striped(true)
            .show(ui, |ui| {
                ui.heading("Engine speed");
                ui.add(
                    egui::DragValue::new(&mut state.engine_speed)
                        .speed(25)
                        .clamp_range(0_u16..=7000)
                        .suffix(" rpm"),
                );

                ui.checkbox(&mut state.engine_speed_fault, "Fault");
                ui.end_row();

                ui.heading("Vehicle speed");
                ui.add(
                    egui::DragValue::new(&mut state.vehicle_speed)
                        .clamp_range(0_u16..=250)
                        .suffix(" km/h"),
                );

                ui.checkbox(&mut state.vehicle_speed_fault, "Fault");
                ui.end_row();

                ui.heading("Boost");
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut state.boost, 0..=255).show_value(false));
                    ui.label(format!("{:.0}%", state.get_boost_percentage() * 100.0));
                });
                ui.end_row();

                ui.heading("Coolant temperature 1");
                ui.add(
                    egui::DragValue::new(&mut state.coolant_temperature_1)
                        .clamp_range(0_u16..=150)
                        .suffix("°C"),
                );
                ui.checkbox(&mut state.coolant_temperature_1_fault, "Fault");
                ui.end_row();

                ui.heading("Coolant temperature 2");
                ui.add(
                    egui::DragValue::new(&mut state.coolant_temperature_2)
                        .clamp_range(0_u16..=150)
                        .suffix("°C"),
                );
                ui.checkbox(&mut state.coolant_temperature_2_fault, "Fault");
                ui.end_row();

                ui.heading("Fuel level");
                ui.add(
                    egui::DragValue::new(&mut state.fuel_level)
                        .clamp_range(0_u16..=700)
                        .suffix("cc"),
                );
                ui.checkbox(&mut state.fuel_level_fault, "Fault");
                ui.end_row();
            });
    }
}

fn main() {
    // let socket = socketcan::CANSocket::open("vcan1").unwrap();
    // let data: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    // let frame = socketcan::CANFrame::new(0x01, &data, false, false).unwrap();
    // socket.write_frame(&frame).unwrap();

    // const FRAME: [u8; 8] = [0x30, 0x03, 0xFB, 0x34, 0x93, 0x00, 0x64, 0x00];

    // let (rest, val) = can::t7::Engine::from_bytes((&FRAME, 0)).unwrap();
    // println!("{:?}", rest);
    // println!("{:?}", val);

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "mui com",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(MiuComApp::default())
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn it_gets_boost_as_percentage() {
        let mut state = MiuState::default();

        assert_eq!(state.boost, 0);
        assert_eq!(state.get_boost_percentage(), 0.0);

        state.boost = 255;
        assert_eq!(state.get_boost_percentage(), 1.0);

        state.boost = 100;
        assert_approx_eq!(state.get_boost_percentage(), 0.39215687);
    }

    #[test]
    fn it_sets_boost_as_percentage() {
        let mut state = MiuState::default();

        state.set_boost_percentage(0.0);
        assert_eq!(state.boost, 0);

        state.set_boost_percentage(1.0);
        assert_eq!(state.boost, 255);

        state.set_boost_percentage(0.1);
        assert_eq!(state.boost, 25);

        state.set_boost_percentage(0.5);
        assert_eq!(state.boost, 127);
    }
}
