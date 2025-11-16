

pub trait Subsystem<Config, Req, Res> {
    fn new(config: Config) -> Self;
    fn update(&mut self, config: Config) {}
    fn run(&mut self, req: Req) -> Res;
    fn reset(&mut self) {}
}