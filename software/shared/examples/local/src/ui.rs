use std::time::Duration;

use eframe::egui;
use egui::mutex::Mutex;
use egui_async::{Bind, EguiAsyncPlugin};
use shared::utils::percentage::Percentage;

use crate::wrappers::core::{CurrentCarState, CurrentOutsideState, get_car_state, get_outside_state, get_req_throttle, update_req_throttle};

struct MyApp {
    /// The Bind struct holds the state of our async operation.
    repeat_updator: Bind<(CurrentCarState, CurrentOutsideState), ()>,

    car_state: Option<CurrentCarState>,
    outside_state: Option<CurrentOutsideState>,
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
            repeat_updator: Bind::new(false), // Same as Bind::default()
            throttle_req: Percentage::zero().to_fractional(),
            car_state: None,
            outside_state: None,
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
            let local_perct = Percentage::from_fractional(self.throttle_req/100.0);
            self.repeat_updator.request_every( move || async move {
                update_req_throttle(local_perct).await;
                let car_state = get_car_state().await;
                let outside_state = get_outside_state().await;
                Ok((car_state, outside_state))
            }, Duration::from_millis(50));

            if let Some(Ok((car_state, outside_state))) = self.repeat_updator.read() {
                {
                    let slider_resp = ui.add(egui::Slider::new(&mut self.throttle_req, 0.0..=100.0).text("Throttle Req"));
                    ui.label(format!("We got user throttle req: {:?} and ecu throttle req: {:?}", outside_state.throttle.to_int(), car_state.throttle.to_int()));
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