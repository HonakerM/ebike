use eframe::egui;
use egui_async::{Bind, EguiAsyncPlugin};
use shared::utils::percentage::Percentage;

use crate::wrappers::core::{CurrentCarState, CurrentOutsideState, get_car_state, get_outside_state, get_req_throttle, update_req_throttle};

struct MyApp {
    /// The Bind struct holds the state of our async operation.
    car_state_bind: Bind<CurrentCarState, ()>,
    outside_state_bind: Bind<CurrentOutsideState, ()>,
    update_state_bind: Bind<(), ()>,

    // state
    throttle_req: f32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            // We initialize the Bind and tell it to not retain data
            // if it's not visible for a frame.
            // If set to true, this will retain data even as the
            // element goes undrawn.
            car_state_bind: Bind::new(false), // Same as Bind::default()
            outside_state_bind: Bind::new(false), // Same as Bind::default()
            throttle_req: Percentage::zero().to_fractional(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // This registers the plugin that drives the async event loop.
        // It's idempotent and cheap to call on every frame.
        ctx.plugin_or_default::<EguiAsyncPlugin>(); // <-- REQUIRED

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Async Data Demo");
            ui.add_space(10.0);

            // Request if `data_bind` is None and idle
            // Otherwise, just read it
           if let Some(res) = self.car_state_bind.read_or_request(|| async {
                let car_state = get_car_state().await;
                Ok(car_state)
            }) {
                let car_state = res.unwrap();
                if let Some(res) = self.outside_state_bind.read_or_request(|| async {
                    let current_outside_state = get_outside_state().await;
                    Ok(current_outside_state)
                }) {
                    let current_outside_state = res.unwrap();

                    let slider_resp = ui.add(egui::Slider::new(&mut self.throttle_req, 0.0..=100.0).text("Throttle Req"));
                    if slider_resp.changed() {
                        self.update_state_bind.read_or_request(|| async { 
                            update_req_throttle(Percentage::from_fractional(self.throttle_req/100.0)).await;
                            Ok(())
                        });
                    }
                    ui.label(format!("We got user throttle req: {:?} and ecu throttle req: {:?}", current_outside_state.throttle, car_state.throttle));

                }
            } else {
                ui.label("Getting state...");
                ui.spinner();
            }
        });
    }
}

// Boilerplate
pub fn run() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "egui-async example",
        native_options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}