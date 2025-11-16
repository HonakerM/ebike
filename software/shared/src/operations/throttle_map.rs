use crate::utils::percentage::Percentage;

// aggressive throttle application first
fn level_0(req: Percentage) -> Percentage {
    ((Into::<f32>::into(req)).powf(0.5) * 10.0).into()
}

// Linear curve over 1secs to full speed
fn level_1(req: Percentage) -> Percentage {
    req
}

// soft throttle application
fn level_2(req: Percentage) -> Percentage {
    ((Into::<f32>::into(req)).powf(2.0) * 0.01).into()
}

#[derive(Debug, Clone, Copy)]
pub enum ThottleMapMode {
    Level0(),
    Level1(),
    Level2(),
}

impl Into<u8> for ThottleMapMode {
    fn into(self) -> u8 {
        match self {
            ThottleMapMode::Level0() => 0,
            ThottleMapMode::Level1() => 1,
            ThottleMapMode::Level2() => 2,
        }
    }
}

impl From<u8> for ThottleMapMode {
    fn from(value: u8) -> Self {
        if value == 2 {
            ThottleMapMode::Level2()
        } else if value == 1 {
            ThottleMapMode::Level1()
        } else {
            ThottleMapMode::Level0()
        }
    }
}

pub struct ThottleMap {
    pub mode: ThottleMapMode,
}

impl ThottleMap {
    pub fn new(mode: ThottleMapMode) -> Self {
        ThottleMap { mode }
    }
    pub fn update_mode(&mut self, mode: ThottleMapMode) {
        self.mode = mode;
    }
    pub fn run_algo(&self, req: Percentage) -> Percentage {
        match self.mode {
            ThottleMapMode::Level0() => level_0(req),
            ThottleMapMode::Level1() => level_1(req),
            ThottleMapMode::Level2() => level_2(req),
        }
    }
}
