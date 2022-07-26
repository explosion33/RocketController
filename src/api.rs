use rocket::{
    self,
    serde::{json::Json},
    Shutdown,
    State,
    Config,
};
use std::sync::{Arc, Mutex};

pub struct Data {
    pub altitude: Vec<(f32, f32)>,
}

impl Data {
    pub fn new() -> Data {
        Data {altitude: vec![(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)]}
    }
}

pub type TData = Arc<Mutex<Data>>;


#[rocket::get("/api/<field>/<points>")]
fn handle_api(state: &State<TData>, field: &str, points: i32) -> Json<Vec<(f32, f32)>> {
    let data = Arc::clone(&state);
    let data = data.lock().expect("could not lock mutex");
    
    let is_neg: bool = points < 0;
    let points: i32 = if is_neg {points*-1} else {points};
    let points: usize = points as usize;

    println!("{}, -: {}", points, is_neg);

    if field == "alt" {
        if points > data.altitude.len() && !is_neg {
            return Json(vec![]);
        }

        let res: Vec<(f32, f32)>;
        if is_neg {
            let index: usize;
            if points > data.altitude.len() {
                index = 0;
            }
            else {
                index = data.altitude.len() - points;
            }

            res = data.altitude[index..].to_vec();
           
        }
        else {
            res = data.altitude[points as usize..].to_vec();
        }

        return Json(res);

    }


    Json(vec![])
}

#[rocket::get("/cmd/<cmd>")]
fn handle_cmd(cmd: &str) -> &'static str {
    println!("{}", cmd);
    
    if cmd == "arm" {
        println!("wrong arm");
    }


    ""
}

#[rocket::get("/cmd/arm")]
fn shutdown(shutdown: Shutdown) -> &'static str {
    shutdown.notify();
    return "arming";
}


pub fn start_api(data: TData) {
    //let data = Data {altitude: vec![(0.0,0.0), (1.0, 1.0), (2.0, 2.0), (3.0, 3.0)]};
    //let thread_data: TData = Arc::new(Mutex::new(data));


    /*let res = rocket::build()
        .mount("/", rocket::routes![handle_api, handle_cmd, shutdown])
        .manage(Arc::clone(&DATA))
        .launch()
        .await;

    
    match res {
        Ok(_) => {},
        Err(_) => {panic!("could not start api")},
    };*/

    rocket::tokio::runtime::Builder::new_multi_thread()
        .worker_threads(Config::from(Config::figment()).workers)
        // NOTE: graceful shutdown depends on the "rocket-worker" prefix.
        .thread_name("rocket-worker-thread")
        .enable_all()
        .build()
        .expect("create tokio runtime")
        .block_on(async move {
            let _ = rocket::build()
            .mount("/", rocket::routes![handle_api, handle_cmd, shutdown])
            .manage(Arc::clone(&data))
            .launch()
            .await;
        })
}