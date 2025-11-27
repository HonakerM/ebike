use std::time::Duration;

use eframe::egui;
use egui::mutex::Mutex;
use egui_async::{Bind, EguiAsyncPlugin};
use shared::utils::percentage::Percentage;

use crate::simulation::car::CarState;
use crate::{
    simulation::car,
    wrappers::{
        core::{
            CurrentOutsideState, get_car_state, get_outside_state, get_req_throttle,
            update_req_throttle,
        },
        fcu::FcuUiComponent,
    },
};
struct MyApp {
    /// The Bind struct holds the state of our async operation.
    repeat_updator: Bind<(CarState, CurrentOutsideState), ()>,
    fcu_component: FcuUiComponent,

    car_state: Option<CarState>,
    outside_state: Option<CurrentOutsideState>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            // We initialize the Bind and tell it to not retain data
            // if it's not visible for a frame.
            // If set to true, this will retain data even as the
            // element goes undrawn.
            repeat_updator: Bind::new(false), // Same as Bind::default()
            fcu_component: FcuUiComponent::default(),
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

        ctx.set_zoom_factor(5.0);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Bike Control Demo");
            ui.add_space(10.0);

            self.repeat_updator.request_every(
                move || async move {
                    let car_state = get_car_state().await;
                    let outside_state = get_outside_state().await;
                    Ok((car_state, outside_state))
                },
                Duration::from_millis(50),
            );

            if let Some(Ok((car_state, outside_state))) = self.repeat_updator.read() {
                self.car_state = Some(car_state.clone());
                self.outside_state = Some(outside_state.clone());
                {
                    self.fcu_component.ui(ui, car_state);
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
