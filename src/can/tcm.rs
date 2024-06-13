use deku::prelude::*;
use socketcan::CanFrame;
use socketcan::Frame;

#[derive(Debug)]
pub enum Gear {
    Unknown,
    Park,
    Reverse,
    Neutral,
    Drive,
    Limit1,
    Limit2,
    Limit3,
}

impl From<u8> for Gear {
    fn from(id: u8) -> Self {
        match id {
            1 => Gear::Park,
            2 => Gear::Reverse,
            3 => Gear::Neutral,
            4 => Gear::Drive,
            5 => Gear::Limit1,
            6 => Gear::Limit2,
            7 => Gear::Limit3,
            _ => Gear::Unknown,
        }
    }
}

impl From<Gear> for u8 {
    fn from(gear: Gear) -> Self {
        match gear {
            Gear::Park => 1,
            Gear::Reverse => 2,
            Gear::Neutral => 3,
            Gear::Drive => 4,
            Gear::Limit1 => 5,
            Gear::Limit2 => 6,
            Gear::Limit3 => 7,
            Gear::Unknown => 0,
        }
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct TransmissionStatus {
    // CanInRaw.X_ActualGearFault
    #[deku(pad_bits_before = "2", bits = 2)]
    pub actual_gear_fault: u8,

    // CanInRaw.X_GearLeverFault
    #[deku(bits = 2)]
    pub gear_lever_fault: u8,

    // CanInRaw.X_ActualGear
    #[deku(pad_bits_before = "2")]
    pub actual_gear: u8,

    // CanInRaw.X_GearLever
    pub gear_lever: u8,

    // Check gearbox
    #[deku(bits = 1)]
    pub check_gearbox: u8,

    // ActualIn.ST_TCMSport
    #[deku(bits = 1)]
    pub sport: u8,

    // TCMWinter
    #[deku(bits = 1)]
    pub winter: u8,

    // CanInRaw.ST_Interv bit 6
    #[deku(bits = 1)]
    pub unknown: u8,

    // ActualIn.ST_TCMFreezeFrameReq
    #[deku(pad_bits_before = "2", bits = 1)]
    pub freeze_frame_request: u8,

    // ActualIn.ST_CheckEngine
    #[deku(bits = 1)]
    pub check_engine: u8,

    // CanInRaw.ST_TCMCSLU
    #[deku(pad_bits_before = "1", bits = 1)]
    pub tcm_cslu: u8,

    // CanInRaw.field_33
    #[deku(pad_bits_before = "5")]
    pub unknown2: u8,
}

impl TransmissionStatus {
    pub const CAN_ID: u32 = 0x3E0;
}

impl TryInto<CanFrame> for TransmissionStatus {
    type Error = DekuError;

    fn try_into(self) -> Result<CanFrame, Self::Error> {
        // Constructing the frame can fail if the id is invalid. In this case the id is
        // static and known valid, so unwrapping is acceptable.
        Ok(CanFrame::from_raw_id(Self::CAN_ID, &self.to_bytes()?).unwrap())
    }
}
