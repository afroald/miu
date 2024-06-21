use deku::prelude::*;
use socketcan::CanFrame;
use socketcan::Frame;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct EngineSpeedAndThrottle {
    /// FaultCANOut.n_Engine
    #[deku(pad_bits_before = "2", bits = 2)]
    pub speed_fault: u8,

    /// FaultCANOut.m_and_p_AirInlet
    #[deku(bits = 2)]
    pub air_inlet_fault: u8,

    /// FaultCANOut.Throttle
    #[deku(bits = 2)]
    pub throttle_fault: u8,

    /// Out.n_Engine
    pub speed: u16,

    /// bOut_M_Engine
    pub torque: u8,

    /// bOut_M_MaxAtActualRPM
    pub max_torque_at_rpm: u8,

    /// bOut_X_AccPedal_div10
    pub accelerator_pedal_position: u8,

    /// bOut_X_AccPedal_shr2
    pub accelerator_pedal_position_gradient: u8,

    /// bOut_M_DTI
    pub dti: u8,
}

impl EngineSpeedAndThrottle {
    pub const CAN_ID: u32 = 0x1A0;
}

impl TryInto<CanFrame> for EngineSpeedAndThrottle {
    type Error = DekuError;

    fn try_into(self) -> Result<CanFrame, Self::Error> {
        Ok(CanFrame::from_raw_id(Self::CAN_ID, &self.to_bytes()?)
            .expect("from_raw_id can not fail because the id is static and known valid"))
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct EngineStatus {
    #[deku(pad_bits_before = "2", bits = 2)]
    pub vehicle_speed_fault: u8,

    #[deku(bits = 1)]
    pub brake_light_status: u8,

    #[deku(pad_bits_before = "3")]
    pub actual_gear: u8,

    #[deku(pad_bits_before = "1", bits = 1)]
    pub cruise_active: u8,

    #[deku(bits = 1)]
    pub no_ignition_retard: u8,

    #[deku(bits = 1)]
    pub kick_down: u8,

    #[deku(bits = 1)]
    pub clutch_brake: u8,

    #[deku(bits = 1)]
    pub jerk: u8,

    #[deku(bits = 1)]
    pub brake_light: u8,

    #[deku(bits = 1)]
    pub warm_up_shift_pattern: u8,

    #[deku(bits = 1)]
    pub check_filler_cap: u8,

    #[deku(bits = 1)]
    pub warm_up_cycle: u8,

    #[deku(bits = 1)]
    pub automatic: u8,

    #[deku(bits = 1)]
    pub nc_inhibit: u8,

    #[deku(bits = 1)]
    pub gear_shift_inhibit: u8,

    #[deku(bits = 1)]
    pub ac_relay: u8,

    #[deku(bits = 1)]
    pub e_gas_off: u8,

    #[deku(bits = 1)]
    pub limp_home: u8,

    #[deku(bits = 1)]
    pub check_engine: u8,

    #[deku(bits = 1)]
    pub shift_up: u8,

    #[deku(bits = 1)]
    pub cruise_lamp: u8,

    #[deku(bits = 1)]
    pub rep: u8,

    #[deku(pad_bits_before = "4", bits = 1)]
    pub engine_started: u8,

    #[deku(bits = 1)]
    pub cruise_included: u8,

    #[deku(pad_bits_before = "6")]
    pub engine_type: u8,

    #[deku(bits = 1)]
    pub coast_lu_inhibit: u8,
}

impl EngineStatus {
    pub const CAN_ID: u32 = 0x280;
}

impl TryInto<CanFrame> for EngineStatus {
    type Error = DekuError;

    fn try_into(self) -> Result<CanFrame, Self::Error> {
        Ok(CanFrame::from_raw_id(Self::CAN_ID, &self.to_bytes()?)
            .expect("from_raw_id can not fail because the id is static and known valid"))
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct AirAndCoolant {
    // FaultCANOut.T_CoolingSystem
    #[deku(pad_bits_before = "2", bits = 2)]
    pub coolant_temperature_1_fault: u8,

    // FaultCANOut.T_CoolingSystem
    #[deku(bits = 2)]
    pub coolant_temperature_2_fault: u8,

    // FaultCANOut.p_AirAmbient
    #[deku(bits = 2)]
    pub ambient_air_pressure_fault: u8,

    // bOut_T_Engine_plus40
    pub coolant_temperature_1_plus_40: u8,

    // bOut_T_Engine_plus40
    pub coolant_temperature_2_plus_40: u8,

    // Out.p_AirBarometric
    pub ambient_air_pressure: u16,
}

impl AirAndCoolant {
    pub const CAN_ID: u32 = 0x5C0;
}

impl TryInto<CanFrame> for AirAndCoolant {
    type Error = DekuError;

    fn try_into(self) -> Result<CanFrame, Self::Error> {
        Ok(CanFrame::from_raw_id(Self::CAN_ID, &self.to_bytes()?)
            .expect("from_raw_id can not fail because the id is static and known valid"))
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct FuelConsumptionAndBoost {
    // FaultCANOut.ST_IgnOn
    #[deku(pad_bits_before = "2", bits = 2)]
    pub ignition_on_fault: u8,

    // FaultCANOut.field_1
    #[deku(bits = 2)]
    pub unknown: u8,

    // Out.V_FuelConsumed
    #[deku(pad_bits_before = "2")]
    pub fuel_consumed: u16,

    // Out.X_BoostMeter
    pub boost: u8,
}

impl FuelConsumptionAndBoost {
    pub const CAN_ID: u32 = 0x370;
}

impl TryInto<CanFrame> for FuelConsumptionAndBoost {
    type Error = DekuError;

    fn try_into(self) -> Result<CanFrame, Self::Error> {
        Ok(CanFrame::from_raw_id(Self::CAN_ID, &self.to_bytes()?)
            .expect("from_raw_id can not fail because the id is static and known valid"))
    }
}
