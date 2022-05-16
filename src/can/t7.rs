use deku::prelude::*;

/// Id 0x1A0 sent by T7
#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct Engine {
    /// FaultCANOut.n_Engine
    #[deku(pad_bits_before = "2", bits = 2)]
    speed_fault: u8,

    /// FaultCANOut.m_and_p_AirInlet
    #[deku(bits = 2)]
    air_inlet_fault: u8,

    /// FaultCANOut.Throttle
    #[deku(bits = 2)]
    throttle_fault: u8,

    /// Out.n_Engine
    speed: u16,

    /// bOut_M_Engine
    torque: u8,

    /// bOut_M_MaxAtActualRPM
    max_torque_at_rpm: u8,

    /// bOut_X_AccPedal_div10
    accelerator_pedal_position: u8,

    /// bOut_X_AccPedal_shr2
    accelerator_pedal_position_gradient: u8,

    /// bOut_M_DTI
    dti: u8,
}
