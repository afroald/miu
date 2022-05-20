use deku::DekuContainerRead;
use socketcan;
use std::mem::drop;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::miu;
use super::t7;
use crate::MiuState;

#[derive(Clone)]
pub enum ConnectionState {
    Connected,
    Disconnected,
}

pub struct Reader {
    pub interface: Arc<Mutex<Option<String>>>,
    pub connection_state: Arc<Mutex<ConnectionState>>,
}

impl Reader {
    pub fn new() -> Self {
        Self {
            interface: Arc::new(Mutex::new(None)),
            connection_state: Arc::new(Mutex::new(ConnectionState::Disconnected)),
        }
    }

    pub fn connect(&mut self, interface_to_use: &str, state: Arc<Mutex<MiuState>>) {
        let interface_lock = Arc::clone(&self.interface);
        let mut interface = interface_lock.lock().unwrap();
        *interface = Some(interface_to_use.to_string());
        drop(interface);

        let connection_state_lock = Arc::clone(&self.connection_state);

        thread::spawn(move || {
            let interface = interface_lock.lock().unwrap();

            if let Some(interface_str) = &*interface {
                let socket = socketcan::CANSocket::open(&*interface_str).unwrap();
                socket.set_read_timeout(Duration::from_secs(1)).unwrap();
                drop(interface);

                let mut connection_state = connection_state_lock.lock().unwrap();
                *connection_state = ConnectionState::Connected;
                drop(connection_state);

                loop {
                    match socket.read_frame() {
                        Ok(frame) => match frame.id() {
                            t7::Engine::CAN_ID => {
                                if let Ok((_, engine)) = t7::Engine::from_bytes((frame.data(), 0)) {
                                    let mut state = state.lock().unwrap();
                                    state.engine_speed_fault = engine.speed_fault == 1;
                                    state.engine_speed = engine.speed;
                                }
                            }
                            t7::AirAndCoolant::CAN_ID => {
                                if let Ok((_, air_and_coolant)) =
                                    t7::AirAndCoolant::from_bytes((frame.data(), 0))
                                {
                                    let mut state = state.lock().unwrap();
                                    state.coolant_temperature_1 =
                                        air_and_coolant.coolant_temperature_plus_40_1 - 40;
                                    state.coolant_temperature_1_fault =
                                        air_and_coolant.coolant_temperature_fault_1 == 1;
                                    state.coolant_temperature_2 =
                                        air_and_coolant.coolant_temperature_plus_40_2 - 40;
                                    state.coolant_temperature_2_fault =
                                        air_and_coolant.coolant_temperature_fault_2 == 1;
                                }
                            }
                            t7::FuelConsumptionAndBoost::CAN_ID => {
                                if let Ok((_, fuel_consumption_and_boost)) =
                                    t7::FuelConsumptionAndBoost::from_bytes((frame.data(), 0))
                                {
                                    let mut state = state.lock().unwrap();
                                    state.boost = fuel_consumption_and_boost.boost;
                                }
                            }
                            miu::VehicleSpeed::CAN_ID => {
                                if let Ok((_, vehicle_speed)) =
                                    miu::VehicleSpeed::from_bytes((frame.data(), 0))
                                {
                                    let mut state = state.lock().unwrap();
                                    state.vehicle_speed = vehicle_speed.vehicle_speed;
                                    state.vehicle_speed_fault =
                                        vehicle_speed.vehicle_speed_fault == 1;
                                }
                            }
                            miu::FuelLevel::CAN_ID => {
                                if let Ok((_, fuel_level)) =
                                    miu::FuelLevel::from_bytes((frame.data(), 0))
                                {
                                    let mut state = state.lock().unwrap();
                                    state.fuel_level = fuel_level.fuel_level;
                                    state.fuel_level_fault = fuel_level.fuel_level_fault == 1;
                                }
                            }
                            _ => {}
                        },
                        Err(error) => println!("Reading frame failed: {:?}", error),
                    }

                    let interface = interface_lock.lock().unwrap();
                    if let None = *interface {
                        let mut connection_state = connection_state_lock.lock().unwrap();
                        *connection_state = ConnectionState::Disconnected;
                        break;
                    }
                }
            }
        });
    }

    pub fn disconnect(&self) {
        let mut interface = self.interface.lock().unwrap();
        *interface = None;
    }
}
