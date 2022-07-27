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


const FIRE: u8 = 23;
const CONT: u8 = 24;
const BARO_CONFIG_PATH: &str = "baro.conf";


fn main() {
    let data = api::Data::new();
    let thread_data: api::TData = Arc::new(Mutex::new(data));
    let collect = Arc::clone(&thread_data);
    thread::spawn(move || {
        let mut barometer = Baro::new(BARO_CONFIG_PATH);
        let mut main_ign = Igniter::new(FIRE, CONT);

        let start = SystemTime::now();
        loop {
            {
                let dt = SystemTime::now().duration_since(start).expect("time fucked up").as_secs_f32();
                let mut data = collect.lock().unwrap();
                match barometer.get_alt() {
                    Ok(n) => {
                        data.altitude.push((dt, n));
                    },
                    Err(_) => {},
                };
                data.cont_main.push((dt, main_ign.has_continuity() as i8 as f32));
            }
            //allow other threads a chance to lock mutex
            thread::sleep(Duration::from_millis(50)); 
        }
    });


    start_api(Arc::clone(&thread_data));

    /*let mut main_ign = Igniter::new(FIRE, CONT);
    println!("{}", main_ign.has_continuity());
    main_ign.fire_async().join();

    let mut barometer = Baro::new(BARO_CONFIG_PATH);

    loop {
        match barometer.get_alt() {
            Ok(n) => {
                println!("alt: {n}");
            },
            Err(_) => {},
        };
    }*/
}
