use pid_lite::Controller;
use std::thread;
use std::time::Duration;

static mut var: f64 = 1000.0;

fn measure() -> f64 {
    // Your sensor / process reading here
    // For example, pretend we're always at 50.0
    unsafe { var }
}

fn apply_correction(correction: f64) {
    // Apply the PID output to your system
    println!("Applying correction: {:.2}", correction);
    unsafe {
        var += correction + 0.1;
    }
}

fn main() {
    let target: f64 = 600.0;
    let mut controller = Controller::new(target, 0.1, 0.0, 0.0);

    loop {
        let current = measure();
        println!("Have Desired: {:.2} with Current: {:.2}", target, current);
        let correction = controller.update_elapsed(current, Duration::from_millis(100));
        apply_correction(correction);
        thread::sleep(Duration::from_millis(100));
    }
}
