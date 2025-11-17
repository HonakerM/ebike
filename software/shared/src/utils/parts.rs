#[derive(Debug, Clone, Copy)]
pub enum Wheel {
    Rear,
    Front,
}

impl Into<u8> for Wheel {
    fn into(self) -> u8 {
        match self {
            Wheel::Rear => 0,
            Wheel::Front => 1,
        }
    }
}
impl From<u8> for Wheel {
    fn from(value: u8) -> Self {
        if value == 1 {
            Wheel::Front
        } else {
            Wheel::Rear
        }
    }
}
