use socketcan::tokio::CanSocket;
use socketcan::Frame;
use std::time::Duration;
use tokio::sync::{mpsc, watch};
use tokio::time;

use crate::miu_state;

pub mod interfaces;
mod miu;
mod t7;
pub mod tcm;

pub enum Command {
    Connect(String, watch::Receiver<miu_state::MiuState>),
    Disconnect,
}

pub type CommandSender = mpsc::Sender<Command>;
type CommandReceiver = mpsc::Receiver<Command>;

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Connected,
    Disconnected,
}

impl Default for State {
    fn default() -> Self {
        Self::Disconnected
    }
}

type StateSender = watch::Sender<State>;
pub type StateReceiver = watch::Receiver<State>;

#[allow(dead_code)]
#[derive(Debug)]
enum CanError {
    IO(std::io::Error),
    MiuStateChannelClosed,
    Serialization(deku::error::DekuError),
    SocketCan(socketcan::Error),
}

impl From<std::io::Error> for CanError {
    fn from(error: std::io::Error) -> Self {
        Self::IO(error)
    }
}

impl From<deku::error::DekuError> for CanError {
    fn from(error: deku::error::DekuError) -> Self {
        Self::Serialization(error)
    }
}

impl From<socketcan::Error> for CanError {
    fn from(error: socketcan::Error) -> Self {
        Self::SocketCan(error)
    }
}

const UPDATE_RATE_HZ: u64 = 20;

/// An infinite task that sends miu state updates at a rate of 20Hz.
///
/// Note: This task runs forever but it can safely be aborted. The socket will be closed normally
/// when it goes out of scope.
async fn broadcast_state(
    interface: String,
    mut miu_state: watch::Receiver<miu_state::MiuState>,
) -> Result<(), CanError> {
    tracing::info!("broadcasting miu state on can bus");

    let socket = CanSocket::open(&interface)?;
    let mut state = *miu_state.borrow_and_update();
    let mut interval = time::interval(Duration::from_millis(1000 / UPDATE_RATE_HZ));

    loop {
        tokio::select! {
            result = miu_state.changed() => {
                if result.is_err() {
                    tracing::info!("ending miu state broadcast because state channel closed");
                    return Err(CanError::MiuStateChannelClosed);
                }

                state = *miu_state.borrow_and_update();
            }

            _ = interval.tick() => {
                let engine = t7::EngineSpeedAndThrottle {
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
                tracing::debug!("sending can message: {:?}", engine);
                socket.write_frame(engine.try_into()?)?.await?;

                let engine_status = t7::EngineStatus {
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
                tracing::debug!("sending can message: {:?}", engine_status);
                socket.write_frame(engine_status.try_into()?)?.await?;

                let air_and_coolant = t7::AirAndCoolant {
                    coolant_temperature_1_fault: state.coolant_temperature_fault.into(),
                    coolant_temperature_2_fault: state.coolant_temperature_fault.into(),
                    ambient_air_pressure_fault: false.into(),
                    coolant_temperature_1_plus_40: state.coolant_temperature + 40,
                    coolant_temperature_2_plus_40: state.coolant_temperature + 40,
                    ambient_air_pressure: 0,
                };
                tracing::debug!("sending can message: {:?}", air_and_coolant);
                socket.write_frame(air_and_coolant.try_into()?)?.await?;

                let fuel_consumption_and_boost = t7::FuelConsumptionAndBoost {
                    ignition_on_fault: false.into(),
                    unknown: 0,
                    fuel_consumed: 0,
                    boost: state.boost,
                };
                tracing::debug!("sending can message: {:?}", fuel_consumption_and_boost);
                socket.write_frame(fuel_consumption_and_boost.try_into()?)?.await?;

                let transmission_status = tcm::TransmissionStatus {
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
                tracing::debug!("sending can message: {:?}", transmission_status);
                socket.write_frame(transmission_status.try_into()?)?.await?;

                // This turns off the abs / traction control warning lights
                let can_frame = socketcan::CanFrame::from_raw_id(0x318, &[0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
                tracing::debug!("sending can message: {:?}", can_frame);
                socket.write_frame(can_frame)?.await?;

                interval.tick().await;
            }
        }
    }
}

#[derive(Debug)]
pub enum CanClientError {
    WorkerStopped,
}

impl From<mpsc::error::SendError<Command>> for CanClientError {
    fn from(_: mpsc::error::SendError<Command>) -> Self {
        Self::WorkerStopped
    }
}

impl From<watch::error::RecvError> for CanClientError {
    fn from(_: watch::error::RecvError) -> Self {
        Self::WorkerStopped
    }
}

pub struct CanClient {
    runtime: tokio::runtime::Handle,
    command: CommandSender,
    connection_state: StateReceiver,
}

impl CanClient {
    pub fn connect(
        &self,
        interface: String,
        miu_state: watch::Receiver<miu_state::MiuState>,
    ) -> Result<(), CanClientError> {
        let command = self.command.clone();
        self.runtime
            .block_on(async { command.send(Command::Connect(interface, miu_state)).await })?;
        Ok(())
    }

    pub fn disconnect(&self) -> Result<(), CanClientError> {
        let command = self.command.clone();

        self.runtime
            .block_on(async { command.send(Command::Disconnect).await })?;

        Ok(())
    }

    pub fn state(&self) -> Result<State, CanClientError> {
        self.connection_state.has_changed()?;
        Ok(*self.connection_state.borrow())
    }
}

pub struct CanTask {
    command: CommandReceiver,
    connection_state: StateSender,
}

impl CanTask {
    pub async fn run(&mut self) {
        tracing::info!("starting can task");

        // Spawn an empty future to make sure the variable always has a valid value
        let mut broadcast_task = tokio::spawn(async {});

        loop {
            match self.command.recv().await {
                Some(Command::Connect(interface, miu_state)) => {
                    tracing::info!("received connect command");

                    broadcast_task.abort();

                    let connection_state = self.connection_state.clone();
                    broadcast_task = tokio::spawn(async move {
                        let result = broadcast_state(interface, miu_state).await;
                        tracing::warn!("broadcasting miu state ended: {:?}", result);

                        // If this send fails the client has gone out of scope, in which case this
                        // state update is not relevant, so we can just ignore the error.
                        let _ = connection_state.send(State::Disconnected);
                    });
                    let _ = self.connection_state.send(State::Connected);
                }
                Some(Command::Disconnect) => {
                    tracing::info!("received disconnect command, aborting broadcast task");

                    broadcast_task.abort();

                    // If this send fails the client has gone out of scope, in which case this
                    // state update is not relevant, so we can just ignore the error.
                    let _ = self.connection_state.send(State::Disconnected);
                }
                None => {
                    // The command channel has closed which means the client has gone out of scope,
                    // so we can end because there is nothing left to do.
                    tracing::info!("ending task because command channel closed");
                    broadcast_task.abort();
                    break;
                }
            }
        }

        tracing::info!("can task ended");
    }
}

pub fn task(runtime: tokio::runtime::Handle) -> (CanClient, CanTask) {
    // A buffer size of 8 is arbitrary. Messages should not come in faster than they are handled
    // because it's really hard to click that fast, but we'll see how it goes.
    let (command_sender, command_receiver) = mpsc::channel::<Command>(8);
    let (state_sender, state_receiver) = watch::channel(State::default());

    let client = CanClient {
        runtime,
        command: command_sender,
        connection_state: state_receiver,
    };

    let task = CanTask {
        command: command_receiver,
        connection_state: state_sender,
    };

    (client, task)
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn client_returns_err_when_task_died() {
        let runtime = tokio::runtime::Runtime::new().expect("unable to create tokio runtime");
        let (client, task) = super::task(runtime.handle().clone());

        drop(task);

        assert!(client.state().is_err());

        runtime.shutdown_background();
    }

    #[tokio::test]
    async fn task_ends_when_client_is_dropped() {
        let runtime = tokio::runtime::Runtime::new().expect("unable to create tokio runtime");
        let (client, mut task) = super::task(runtime.handle().clone());

        let handle = tokio::spawn(async move { task.run().await });

        drop(client);

        assert!(handle.await.is_ok());

        runtime.shutdown_background();
    }
}
