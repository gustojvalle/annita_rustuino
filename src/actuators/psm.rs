use rbd_dimmer::wait_zero_crossing;
use std::time::{Duration, Instant};

// Global state for CPS
static mut CPS_STATE: Option<CPSState> = None;

struct CPSState {
    last_time: Instant,
    click_counter: u32,
}

pub fn initialize() {
    unsafe {
        CPS_STATE = Some(CPSState {
            last_time: Instant::now(),
            click_counter: 0,
        });
    }
}

fn update_clicks() {
    unsafe {
        if let Some(state) = &mut CPS_STATE {
            wait_zero_crossing().unwrap();
            state.click_counter += 1;
        }
    }
}

pub fn calculate_cps() -> i32 {
    unsafe {
        if let Some(state) = &mut CPS_STATE {
            let start_time = Instant::now();
            let duration = Duration::from_millis(100);
            state.last_time = start_time;
            state.click_counter = 0;

            while Instant::now().duration_since(start_time) < duration {
                update_clicks(); // Update clicks during the measurement period

                // Optional: Adjust sleep duration to avoid busy-waiting
                std::thread::sleep(Duration::from_millis(10));
            }

            let elapsed = Instant::now().duration_since(start_time).as_secs_f32();
            let cps = if elapsed > 0.0 {
                state.click_counter as i32 / elapsed as i32
            } else {
                0
            };

            cps // Return CPS value
        } else {
            0
        }
    }
}
