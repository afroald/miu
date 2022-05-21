use crate::can;
use crate::miu_state;
use deku::DekuContainerRead;
use deku::DekuContainerWrite;
use eframe::egui;
use std::sync::{atomic, Arc, Mutex};
use std::thread;
use std::time;

#[derive(Debug, PartialEq)]
enum Mode {
    Display,
    Control,
}

#[derive(Clone, Debug, PartialEq)]
enum ConnectionState {
    Connected,
    Disconnected,
}

pub struct MiuComApp {
    ui_context: egui::Context,
    mode: Mode,
    can_interfaces: can::interfaces::CanInterfaces,
    selected_can_interface: Option<String>,
    connection_state: Arc<Mutex<ConnectionState>>,
    please_disconnect: Arc<atomic::AtomicBool>,
    state: Arc<Mutex<miu_state::MiuState>>,
}

impl MiuComApp {
    pub fn new(ui_context: egui::Context) -> Self {
        Self {
            ui_context,
            mode: Mode::Control,
            can_interfaces: can::interfaces::CanInterfaces::new(),
            selected_can_interface: None,
            connection_state: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            please_disconnect: Arc::new(atomic::AtomicBool::new(false)),
            state: Arc::new(Mutex::new(miu_state::MiuState::default())),
        }
    }
}

impl eframe::App for MiuComApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top-bar").show(ctx, |ui| {
            self.top_bar(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.mode {
            Mode::Display => self.display_grid_contents(ui),
            Mode::Control => self.control_grid_contents(ui),
        });
    }
}

impl MiuComApp {
    fn start_display_thread(&self) {
        let interface = self.selected_can_interface.clone().unwrap();
        let connection_state_mutex = Arc::clone(&self.connection_state);
        let should_disconnect = Arc::clone(&self.please_disconnect);
        let state_mutex = Arc::clone(&self.state);
        let ui_context = self.ui_context.clone();

        thread::spawn(move || match socketcan::CANSocket::open(&interface) {
            Ok(socket) => {
                socket
                    .set_read_timeout(time::Duration::from_millis(250))
                    .unwrap();

                {
                    let mut connection_state = connection_state_mutex.lock().unwrap();
                    *connection_state = ConnectionState::Connected;
                }

                while !should_disconnect.load(atomic::Ordering::SeqCst) {
                    match socket.read_frame() {
                        Ok(frame) => {
                            let mut state = state_mutex.lock().unwrap();
                            match frame.id() {
                                can::t7::EngineSpeedAndThrottle::CAN_ID => {
                                    if let Ok((_, engine)) =
                                        can::t7::EngineSpeedAndThrottle::from_bytes((
                                            frame.data(),
                                            0,
                                        ))
                                    {
                                        state.engine_speed_fault = engine.speed_fault == 1;
                                        state.engine_speed = engine.speed;
                                    }
                                }
                                can::t7::EngineStatus::CAN_ID => {
                                    if let Ok((_, status)) =
                                        can::t7::EngineStatus::from_bytes((frame.data(), 0))
                                    {
                                        state.check_engine = status.check_engine == 1;
                                        state.cruise = status.cruise_lamp == 1;
                                    }
                                }
                                can::t7::AirAndCoolant::CAN_ID => {
                                    if let Ok((_, air_and_coolant)) =
                                        can::t7::AirAndCoolant::from_bytes((frame.data(), 0))
                                    {
                                        state.coolant_temperature =
                                            air_and_coolant.coolant_temperature_1_plus_40 - 40;
                                        state.coolant_temperature_fault =
                                            air_and_coolant.coolant_temperature_1_fault == 1;
                                    }
                                }
                                can::t7::FuelConsumptionAndBoost::CAN_ID => {
                                    if let Ok((_, fuel_consumption_and_boost)) =
                                        can::t7::FuelConsumptionAndBoost::from_bytes((
                                            frame.data(),
                                            0,
                                        ))
                                    {
                                        state.boost = fuel_consumption_and_boost.boost;
                                    }
                                }
                                can::tcm::TransmissionStatus::CAN_ID => {
                                    if let Ok((_, transmission_status)) =
                                        can::tcm::TransmissionStatus::from_bytes((frame.data(), 0))
                                    {
                                        state.gear_lever = transmission_status.gear_lever;
                                        state.gear_lever_fault =
                                            transmission_status.gear_lever_fault == 1;
                                        state.actual_gear = transmission_status.actual_gear;
                                        state.actual_gear_fault =
                                            transmission_status.actual_gear_fault == 1;
                                        state.check_gearbox =
                                            transmission_status.check_gearbox == 1;
                                        state.sport = transmission_status.sport == 1;
                                        state.winter = transmission_status.winter == 1;
                                    }
                                }
                                can::miu::VehicleSpeed::CAN_ID => {
                                    if let Ok((_, vehicle_speed)) =
                                        can::miu::VehicleSpeed::from_bytes((frame.data(), 0))
                                    {
                                        state.vehicle_speed = vehicle_speed.vehicle_speed;
                                        state.vehicle_speed_fault =
                                            vehicle_speed.vehicle_speed_fault == 1;
                                    }
                                }
                                can::miu::FuelLevel::CAN_ID => {
                                    if let Ok((_, fuel_level)) =
                                        can::miu::FuelLevel::from_bytes((frame.data(), 0))
                                    {
                                        state.fuel_level = fuel_level.fuel_level;
                                        state.fuel_level_fault = fuel_level.fuel_level_fault == 1;
                                    }
                                }
                                _ => {}
                            }
                            ui_context.request_repaint();
                        }
                        Err(error) => {
                            println!("Error reading frame: {}", error);
                        }
                    }
                }

                let mut connection_state = connection_state_mutex.lock().unwrap();
                *connection_state = ConnectionState::Disconnected;
                should_disconnect.store(false, atomic::Ordering::SeqCst);
            }
            Err(error) => {
                println!("Failed to open socket: {}", error);
            }
        });
    }

    fn start_control_thread(&self) {
        let interface = self.selected_can_interface.clone().unwrap();
        let connection_state_mutex = Arc::clone(&self.connection_state);
        let should_disconnect = Arc::clone(&self.please_disconnect);
        let state_mutex = Arc::clone(&self.state);

        thread::spawn(move || match socketcan::CANSocket::open(&interface) {
            Ok(socket) => {
                socket
                    .set_read_timeout(time::Duration::from_millis(250))
                    .unwrap();

                {
                    let mut connection_state = connection_state_mutex.lock().unwrap();
                    *connection_state = ConnectionState::Connected;
                }

                while !should_disconnect.load(atomic::Ordering::SeqCst) {
                    let state = { state_mutex.lock().unwrap().clone() };
                    // 0x1A0: every 10 milliseconds
                    // 0x5C0: every 1000 milliseconds
                    // 0x370: every 100 milliseconds

                    let engine = can::t7::EngineSpeedAndThrottle {
                        speed_fault: state.engine_speed_fault.into(),
                        air_inlet_fault: false.into(),
                        throttle_fault: false.into(),
                        speed: state.engine_speed,
                        torque: 0,
                        max_torque_at_rpm: 0,
                        accelerator_pedal_position: 0,
                        accelerator_pedal_position_gradient: 0,
                        dti: 0,
                    };
                    let can_frame = socketcan::CANFrame::new(
                        can::t7::EngineSpeedAndThrottle::CAN_ID,
                        &engine.to_bytes().unwrap(),
                        false,
                        false,
                    )
                    .unwrap();
                    socket.write_frame(&can_frame).unwrap();

                    let engine_status = can::t7::EngineStatus {
                        vehicle_speed_fault: false.into(),
                        brake_light_status: 0,
                        actual_gear: 0,
                        cruise_active: 0,
                        no_ignition_retard: 0,
                        kick_down: 0,
                        clutch_brake: 0,
                        jerk: 0,
                        brake_light: 0,
                        warm_up_shift_pattern: 0,
                        check_filler_cap: 0,
                        warm_up_cycle: 0,
                        automatic: 1,
                        nc_inhibit: 0,
                        gear_shift_inhibit: 0,
                        ac_relay: 0,
                        e_gas_off: 0,
                        limp_home: 0,
                        check_engine: state.check_engine.into(),
                        shift_up: 0,
                        cruise_lamp: state.cruise.into(),
                        rep: 0,
                        engine_started: 1,
                        cruise_included: 1,
                        engine_type: 146,
                        coast_lu_inhibit: 0,
                    };
                    let can_frame = socketcan::CANFrame::new(
                        can::t7::EngineStatus::CAN_ID,
                        &engine_status.to_bytes().unwrap(),
                        false,
                        false,
                    )
                    .unwrap();
                    socket.write_frame(&can_frame).unwrap();

                    let air_and_coolant = can::t7::AirAndCoolant {
                        coolant_temperature_1_fault: state.coolant_temperature_fault.into(),
                        coolant_temperature_2_fault: state.coolant_temperature_fault.into(),
                        ambient_air_pressure_fault: false.into(),
                        coolant_temperature_1_plus_40: state.coolant_temperature + 40,
                        coolant_temperature_2_plus_40: state.coolant_temperature + 40,
                        ambient_air_pressure: 0,
                    };
                    let can_frame = socketcan::CANFrame::new(
                        can::t7::AirAndCoolant::CAN_ID,
                        &air_and_coolant.to_bytes().unwrap(),
                        false,
                        false,
                    )
                    .unwrap();
                    socket.write_frame(&can_frame).unwrap();

                    let fuel_consumption_and_boost = can::t7::FuelConsumptionAndBoost {
                        ignition_on_fault: false.into(),
                        unknown: 0,
                        fuel_consumed: 0,
                        boost: state.boost,
                    };
                    let can_frame = socketcan::CANFrame::new(
                        can::t7::FuelConsumptionAndBoost::CAN_ID,
                        &fuel_consumption_and_boost.to_bytes().unwrap(),
                        false,
                        false,
                    )
                    .unwrap();
                    socket.write_frame(&can_frame).unwrap();

                    let transmission_status = can::tcm::TransmissionStatus {
                        actual_gear_fault: state.actual_gear_fault.into(),
                        gear_lever_fault: state.gear_lever_fault.into(),
                        actual_gear: state.actual_gear,
                        gear_lever: state.gear_lever,
                        check_gearbox: state.check_gearbox.into(),
                        sport: state.sport.into(),
                        winter: state.winter.into(),
                        unknown: 0,
                        freeze_frame_request: 0,
                        check_engine: 0,
                        tcm_cslu: 0,
                        unknown2: 0,
                    };
                    let can_frame = socketcan::CANFrame::new(
                        can::tcm::TransmissionStatus::CAN_ID,
                        &transmission_status.to_bytes().unwrap(),
                        false,
                        false,
                    )
                    .unwrap();
                    socket.write_frame(&can_frame).unwrap();

                    // This turns of the abc / traction control warning lights
                    let can_frame =
                        socketcan::CANFrame::new(0x318, &[0, 0, 0, 0, 0, 0, 0, 0], false, false)
                            .unwrap();
                    socket.write_frame(&can_frame).unwrap();

                    // let can_frame =
                    //     socketcan::CANFrame::new(0x3E0, &[0, 0, 0, 0, 0, 0, 0, 0], false, false)
                    //         .unwrap();
                    // socket.write_frame(&can_frame).unwrap();

                    thread::sleep(time::Duration::from_millis(100));
                }

                let mut connection_state = connection_state_mutex.lock().unwrap();
                *connection_state = ConnectionState::Disconnected;
                should_disconnect.store(false, atomic::Ordering::SeqCst);
            }
            Err(error) => {
                println!("Failed to open socket: {}", error);
            }
        });
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        let connection_state = { self.connection_state.lock().unwrap().clone() };

        ui.horizontal(|ui| {
            egui::widgets::global_dark_light_mode_switch(ui);

            ui.separator();

            ui.label("Mode:");
            ui.add_enabled_ui(connection_state == ConnectionState::Disconnected, |ui| {
                egui::ComboBox::from_id_source("mode-selector")
                    .selected_text(format!("{:?}", self.mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.mode, Mode::Display, "Display");
                        ui.selectable_value(&mut self.mode, Mode::Control, "Control");
                    });
            });

            ui.separator();

            let interfaces = self.can_interfaces.lock().unwrap();
            ui.label("Bus:");

            match &*interfaces {
                Ok(interfaces) => {
                    ui.add_enabled_ui(connection_state == ConnectionState::Disconnected, |ui| {
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
                    });

                    match connection_state {
                        ConnectionState::Connected => {
                            if ui.button("Disconnect").clicked() {
                                self.please_disconnect.store(true, atomic::Ordering::SeqCst);
                            }
                        }
                        ConnectionState::Disconnected => {
                            if let Some(_) = &self.selected_can_interface {
                                if ui.button("Connect").clicked() {
                                    match self.mode {
                                        Mode::Display => self.start_display_thread(),
                                        Mode::Control => self.start_control_thread(),
                                    }
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
    }

    fn display_grid_contents(&self, ui: &mut egui::Ui) {
        let state = { self.state.lock().unwrap().clone() };

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
                ui.heading(format!("{}°C", state.coolant_temperature));
                ui.label(format!("Fault: {}", state.coolant_temperature_fault));
                ui.end_row();

                ui.heading("Fuel level");
                ui.heading(format!("{} cc", state.fuel_level));
                ui.label(format!("Fault: {}", state.fuel_level_fault));
                ui.end_row();

                ui.heading("Check engine");
                ui.heading(format!("{}", state.check_engine));
                ui.end_row();

                ui.heading("Cruise");
                ui.heading(format!("{}", state.cruise));
                ui.end_row();

                ui.heading("Gear lever");
                ui.heading(format!("{:?}", can::tcm::Gear::from(state.gear_lever)));
                ui.label(format!("Fault: {}", state.gear_lever_fault));
                ui.end_row();

                ui.heading("Actual gear");
                ui.heading(format!("{:?}", can::tcm::Gear::from(state.actual_gear)));
                ui.label(format!("Fault: {}", state.actual_gear_fault));
                ui.end_row();

                ui.heading("Sport");
                ui.heading(format!("{}", state.sport));
                ui.end_row();

                ui.heading("Winter");
                ui.heading(format!("{}", state.winter));
                ui.end_row();

                ui.heading("Check gearbox");
                ui.heading(format!("{}", state.check_gearbox));
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

                // ui.heading("Vehicle speed");
                // ui.add(
                //     egui::DragValue::new(&mut state.vehicle_speed)
                //         .clamp_range(0_u16..=250)
                //         .suffix(" km/h"),
                // );
                // ui.checkbox(&mut state.vehicle_speed_fault, "Fault");
                // ui.end_row();

                ui.heading("Boost");
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut state.boost, 0..=255).show_value(false));
                    ui.label(format!("{:.0}%", state.get_boost_percentage() * 100.0));
                });
                ui.end_row();

                ui.heading("Coolant temperature 1");
                ui.add(
                    egui::DragValue::new(&mut state.coolant_temperature)
                        .clamp_range(0_u16..=150)
                        .suffix("°C"),
                );
                ui.checkbox(&mut state.coolant_temperature_fault, "Fault");
                ui.end_row();

                // ui.heading("Fuel level");
                // ui.add(
                //     egui::DragValue::new(&mut state.fuel_level)
                //         .clamp_range(0_u16..=700)
                //         .suffix("cc"),
                // );
                // ui.checkbox(&mut state.fuel_level_fault, "Fault");
                // ui.end_row();

                ui.heading("Check engine");
                ui.checkbox(&mut state.check_engine, "");
                ui.end_row();

                ui.heading("Cruise");
                ui.checkbox(&mut state.cruise, "");
                ui.end_row();

                ui.heading("Gear lever");
                egui::ComboBox::from_id_source("gear-lever")
                    .selected_text(format!("{:?}", can::tcm::Gear::from(state.gear_lever)))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut state.gear_lever,
                            can::tcm::Gear::Unknown.into(),
                            format!("{:?}", can::tcm::Gear::Unknown),
                        );
                        ui.selectable_value(
                            &mut state.gear_lever,
                            can::tcm::Gear::Park.into(),
                            format!("{:?}", can::tcm::Gear::Park),
                        );
                        ui.selectable_value(
                            &mut state.gear_lever,
                            can::tcm::Gear::Reverse.into(),
                            format!("{:?}", can::tcm::Gear::Reverse),
                        );
                        ui.selectable_value(
                            &mut state.gear_lever,
                            can::tcm::Gear::Neutral.into(),
                            format!("{:?}", can::tcm::Gear::Neutral),
                        );
                        ui.selectable_value(
                            &mut state.gear_lever,
                            can::tcm::Gear::Drive.into(),
                            format!("{:?}", can::tcm::Gear::Drive),
                        );
                        ui.selectable_value(
                            &mut state.gear_lever,
                            can::tcm::Gear::Limit3.into(),
                            format!("{:?}", can::tcm::Gear::Limit3),
                        );
                        ui.selectable_value(
                            &mut state.gear_lever,
                            can::tcm::Gear::Limit2.into(),
                            format!("{:?}", can::tcm::Gear::Limit2),
                        );
                        ui.selectable_value(
                            &mut state.gear_lever,
                            can::tcm::Gear::Limit1.into(),
                            format!("{:?}", can::tcm::Gear::Limit1),
                        );
                    });
                ui.checkbox(&mut state.gear_lever_fault, "Fault");
                ui.end_row();

                ui.heading("Actual gear");
                egui::ComboBox::from_id_source("actual-gear")
                    .selected_text(format!("{:?}", can::tcm::Gear::from(state.actual_gear)))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut state.actual_gear,
                            can::tcm::Gear::Unknown.into(),
                            format!("{:?}", can::tcm::Gear::Unknown),
                        );
                        ui.selectable_value(
                            &mut state.actual_gear,
                            can::tcm::Gear::Park.into(),
                            format!("{:?}", can::tcm::Gear::Park),
                        );
                        ui.selectable_value(
                            &mut state.actual_gear,
                            can::tcm::Gear::Reverse.into(),
                            format!("{:?}", can::tcm::Gear::Reverse),
                        );
                        ui.selectable_value(
                            &mut state.actual_gear,
                            can::tcm::Gear::Neutral.into(),
                            format!("{:?}", can::tcm::Gear::Neutral),
                        );
                        ui.selectable_value(
                            &mut state.actual_gear,
                            can::tcm::Gear::Drive.into(),
                            format!("{:?}", can::tcm::Gear::Drive),
                        );
                        ui.selectable_value(
                            &mut state.actual_gear,
                            can::tcm::Gear::Limit3.into(),
                            format!("{:?}", can::tcm::Gear::Limit3),
                        );
                        ui.selectable_value(
                            &mut state.actual_gear,
                            can::tcm::Gear::Limit2.into(),
                            format!("{:?}", can::tcm::Gear::Limit2),
                        );
                        ui.selectable_value(
                            &mut state.actual_gear,
                            can::tcm::Gear::Limit1.into(),
                            format!("{:?}", can::tcm::Gear::Limit1),
                        );
                    });
                ui.checkbox(&mut state.actual_gear_fault, "Fault");
                ui.end_row();

                ui.heading("Sport");
                ui.checkbox(&mut state.sport, "");
                ui.end_row();

                ui.heading("Winter");
                ui.checkbox(&mut state.winter, "");
                ui.end_row();

                ui.heading("Check gearbox");
                ui.checkbox(&mut state.check_gearbox, "");
                ui.end_row();
            });
    }
}
