use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct VehicleSpeed {
    // CanInRaw.v_Vehicle2Fault
    #[deku(pad_bits_before = "2", bits = 2)]
    pub vehicle_speed_fault: u8,

    // CanInRaw.v_Vehicle2
    #[deku(pad_bits_before = "4")]
    pub vehicle_speed: u16,

    // ActualIn.ST_BoostMeter
    #[deku(bits = 1)]
    pub boost_meter_status: u8,
}

impl VehicleSpeed {
    pub const CAN_ID: u32 = 0x2f0;
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct FuelLevel {
    // CanInRaw.V_FuelTankFault
    #[deku(pad_bits_before = "6", bits = 2)]
    pub fuel_level_fault: u8,

    // CanInRaw.V_FuelTank
    #[deku(pad_bits_before = "32")]
    pub fuel_level: u16,
}

impl FuelLevel {
    pub const CAN_ID: u32 = 0x631;
}
