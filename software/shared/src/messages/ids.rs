use embedded_can::StandardId;

pub const ECU_MESG_ID: StandardId = unsafe { StandardId::new_unchecked(0x01) };
pub const CTL_MESG_ID: StandardId = unsafe { StandardId::new_unchecked(0x02) };
pub const TRS_MESG_ID: StandardId = unsafe { StandardId::new_unchecked(0x03) };
pub const UPD_MESG_ID: StandardId = unsafe { StandardId::new_unchecked(0x04) };
