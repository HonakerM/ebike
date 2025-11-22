use crate::utils::percentage::Percentage;
use micromath::F32Ext;

// aggressive throttle application first
fn level_0(req: Percentage) -> Percentage {
    ((Into::<f32>::into(req)).powf(0.5)).into()
}

// Direct 1 to 1 throttle mapping
fn level_1(req: Percentage) -> Percentage {
    req
}

// soft throttle application
fn level_2(req: Percentage) -> Percentage {
    ((Into::<f32>::into(req)).powf(2.0)).into()
}

#[derive(Debug, Clone, Copy)]
pub enum ThottleMapMode {
    Level0(),
    Level1(),
    Level2(),
}

impl ThottleMapMode { 
    pub fn update(&self, req: Percentage) -> Percentage {
        match self {
            ThottleMapMode::Level0() => {level_0(req)}
            ThottleMapMode::Level1() => {level_1(req)}
            ThottleMapMode::Level2() => {level_2(req)}
        }
    }
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
        self.mode.update(req)
    }
}
