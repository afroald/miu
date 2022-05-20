use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct Engine {
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

impl Engine {
    pub const CAN_ID: u32 = 0x1A0;
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct AirAndCoolant {
    // FaultCANOut.T_CoolingSystem
    #[deku(pad_bits_before = "2", bits = 2)]
    pub coolant_temperature_fault_1: u8,

    // FaultCANOut.T_CoolingSystem
    #[deku(bits = 2)]
    pub coolant_temperature_fault_2: u8,

    // FaultCANOut.p_AirAmbient
    #[deku(bits = 2)]
    pub ambient_air_pressure_fault: u8,

    // bOut_T_Engine_plus40
    pub coolant_temperature_plus_40_1: u8,

    // bOut_T_Engine_plus40
    pub coolant_temperature_plus_40_2: u8,

    // Out.p_AirBarometric
    pub ambient_air_pressure: u16,
}

impl AirAndCoolant {
    pub const CAN_ID: u32 = 0x5C0;
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
