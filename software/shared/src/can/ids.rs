use embedded_can::StandardId;

pub const MCU_MESG_ID: StandardId = unsafe { StandardId::new_unchecked(0x01) };
pub const FCU_MESG_ID: StandardId = unsafe { StandardId::new_unchecked(0x02) };
pub const RCU_MESG_ID: StandardId = unsafe { StandardId::new_unchecked(0x03) };
