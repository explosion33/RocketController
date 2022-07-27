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
    pub cont_main: Vec<(f32, f32)>,
    pub cont_droug: Vec<(f32, f32)>,
}

impl Data {
    pub fn new() -> Data {
        Data {altitude: vec![], cont_main: vec![], cont_droug: vec![]}
    }
}

pub type TData = Arc<Mutex<Data>>;

fn compile_vec(data: &mut Vec<(f32, f32)>, points: usize, is_neg: bool) -> Vec<(f32, f32)> {
    if points > data.len() && !is_neg {
        return vec![];
    }

    let res: Vec<(f32, f32)>;
    if is_neg {
        let index: usize;
        if points > data.len() {
            index = 0;
        }
        else {
            index = data.len() - points;
        }

        res = data[index..].to_vec();
       
    }
    else {
        res = data[points as usize..].to_vec();
    }

    return res;
}

#[rocket::get("/api/<field>/<points>")]
fn handle_api(state: &State<TData>, field: &str, points: i32) -> Json<Vec<(f32, f32)>> {
    let data = Arc::clone(&state);
    let mut data = data.lock().expect("could not lock mutex");
    
    let is_neg: bool = points < 0;
    let points: i32 = if is_neg {points*-1} else {points};
    let points: usize = points as usize;

    println!("{}, -: {}", points, is_neg);

    return match field {
        "alt" => {
            Json(compile_vec(&mut data.altitude, points, is_neg))
        },

        "cont_main" => {
            Json(compile_vec(&mut data.cont_main, points, is_neg))
        },

        "cont_droug" => {
            Json(compile_vec(&mut data.cont_droug, points, is_neg))
        },

        _ => {
            Json(vec![])
        },
        
    }
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