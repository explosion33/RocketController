use crate::igniter::Igniter;
mod igniter;

use crate::baro::Baro;
mod baro;

use crate::api::{start_api};
mod api;

use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;


const M_FIRE: u8 = 23;
const M_CONT: u8 = 22;

const D_FIRE: u8 = 27;
const D_CONT: u8 = 17;

const BARO_CONFIG_PATH: &str = "baro.conf";

//rolling average
const WINDOW_SIZE: usize = 10;
//number of points for baseline average
const BASELINE_ITER: usize = 150;
//change in meters, to count as a significant change
//should be adjusted to overcome noise
const SIG_ALT_CHANGE: f32 = 1.1f32;
const ITER_ABOVE_SIG: u8 = 100;

const START_ALT: f32 = 189f32;

fn pre_flight() {
    // initiate shared memory
    let data = api::Data::new();
    let thread_data: api::TData = Arc::new(Mutex::new(data));
    let collect = Arc::clone(&thread_data); //created then moved
    
    // start and wait for thread to finish
    let handle = thread::spawn(move || {
        api_getter(collect);
    });

    start_api(thread_data);
    handle.join();
}

fn api_getter(thread_data: api::TData) {
    // sensors
    let mut barometer = Baro::new(BARO_CONFIG_PATH);
    let mut main_ign = Igniter::new(M_FIRE, M_CONT);
    let mut droug_ign = Igniter::new(D_FIRE, D_CONT);

    // averaging vars
    let mut window: Vec<f32> = vec![];
    const WINDOW_SIZE: usize = 10;

    //main loop
    let start = SystemTime::now();
    loop {
        let dt = SystemTime::now().duration_since(start).expect("time fucked up").as_secs_f32();
        let mut data = thread_data.lock().unwrap();

        // check if rocket server has been closed
        if !data.is_alive {
            return ();
        }

        // sort through any push commands
        for command in data.cmds.iter() {
            let (cmd, val) = command;
            match cmd.as_str() {
                "config_baro" => {
                    barometer.configure(*val);
                },
                "fire" => {
                    match *val {
                        0f32 => {droug_ign.fire_async();},
                        1f32 => {main_ign.fire_async();},
                        _ => {},
                    };
                },
                _ => {},
            };
        }
        data.cmds.clear();

        // get barometer reading and adjust window
        match barometer.get_alt() {
            Ok(n) => {
                window.push(n);
                if window.len() > WINDOW_SIZE {
                    window.remove(0);
                }
            },
            Err(_) => {},
        };

        // if window is full, push the computed average to api server
        if window.len() == WINDOW_SIZE {
            let mut avg: f32 = 0f32;
            for alt in window.iter() {
                avg += alt;
            }
            avg /= WINDOW_SIZE as f32;

            data.altitude.push((dt, avg));
        }

        // continuity sensor data
        data.cont_main.push((dt, main_ign.has_continuity() as i8 as f32));
        data.cont_droug.push((dt, droug_ign.has_continuity() as i8 as f32));

        //allow other threads a chance to lock mutex
        drop(data);
        thread::sleep(Duration::from_millis(50)); 
    }
}

fn detect_liftoff(barometer: &mut Baro) {
    // get average resting altitude
    let mut base_alt: f32 = 0f32;
    let mut i = 0;
    while i < BASELINE_ITER {
        match barometer.get_alt() {
            Ok(n) => {
                base_alt += n;
                i += 1;
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => {}
        }
    }
    base_alt /= i as f32;
    println!("base-alt: {}", base_alt);

    // detect significant upwards elevation change
    let mut window: Vec<f32> = vec![];
    let mut num_above_sig: u8 = 0;
    loop {
        // rolling average altitudes
        match barometer.get_alt() {
            Ok(n) => {
                window.push(n);
                if window.len() > WINDOW_SIZE {
                    window.remove(0);
                }
            },
            Err(_) => {},
        };

        // if our window is not full skip calculations
        // i.e. first WINDOW_SIZE - 1 values
        if window.len() != WINDOW_SIZE {
            continue;
        }

        // average current window
        let mut avg: f32 = 0f32;
        for alt in window.iter() {
            avg += alt;
        }
        avg /= WINDOW_SIZE as f32;

        // is significantly high
        if avg - base_alt >= SIG_ALT_CHANGE {
            num_above_sig += 1;
            println!("{}/{}", num_above_sig, ITER_ABOVE_SIG);
        }
        else {
            num_above_sig = 0;
        }

        // if above height threshold for given iterations
        if num_above_sig > ITER_ABOVE_SIG {
            return ();
        }
    }
}

fn main() {
    // pre-flight API
    pre_flight();

    // ARMED STAGE
    /*
     * TODO:
     * add second igniter
     * handle push commands for igniter ignition
     * add ascent, descent under droug, and descent under main states
     * add buzzer + external LED support
     * fine tune above constants to remove noise from calculations
     *
     * TESTING:
     * throw test, ensure all stages are met
     * 
    */

    let mut main_ign = Igniter::new(M_FIRE, M_CONT);
    let mut droug_ign = Igniter::new(D_FIRE, D_CONT);    
    let mut barometer = Baro::new(BARO_CONFIG_PATH);

    //detect_liftoff(&mut barometer);

}
