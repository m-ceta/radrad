extern crate m3u8_rs;
extern crate eyre;
extern crate reqwest;
extern crate quick_xml;
extern crate regex;
extern crate base64;
#[macro_use]
extern crate lazy_static;
extern crate actix_web;
#[cfg(target_os = "linux")]
extern crate signal_hook;

use std::sync::RwLock;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
#[cfg(target_os = "linux")]
use signal_hook::consts::signal::*;
#[cfg(target_os = "linux")]
use std::thread;

mod rd_client;
mod stream_player;
mod models;

lazy_static! {
    pub static ref PLAYER: RwLock<stream_player::Player> = {
        RwLock::new(stream_player::Player::new())
    };
}

#[get("/radrad")]
async fn get_radrad() -> impl Responder {
    println!("get_radrad");
    if let Ok(plyr) = PLAYER.read() {
        let res = HttpResponse::Ok().json(models::Station {
            id: plyr.get_station_id(),
            af: plyr.get_option_af(),
            tf: plyr.get_option_tf(),
        });
        return res;
    }
    HttpResponse::NotFound().body("An error occurred while running.")
}

#[post("/radrad/play")]
async fn radrad_play(station: web::Json<models::Station>) -> impl Responder {
    if let Ok(mut plyr) = PLAYER.write() {
        if let Some(sta_id) = &station.id {
            plyr.play(sta_id, station.af, station.tf);
            return HttpResponse::Ok().body("ok");
        }
    }
    HttpResponse::NotFound().body("An error occurred while running.")
}

#[get("/radrad/stop")]
async fn radrad_stop() -> impl Responder {
    if let Ok(mut plyr) = PLAYER.write() {
        plyr.stop();
        return HttpResponse::Ok().body("ok");
    }
    HttpResponse::NotFound().body("An error occurred while running.")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if cfg!(target_os = "linux") {
        let mut signals = signal_hook::iterator::Signals::new(&[SIGINT, SIGQUIT, SIGHUP, SIGTERM, SIGTSTP]).unwrap();
        thread::spawn(move || {
            let mut flag = true;
            for _sig in signals.forever() {
                if flag {
                    if let Ok(mut plyr) = PLAYER.write() {
                        plyr.stop();
                    }
                    flag = false;
                }
            }
        });
    }
    HttpServer::new(|| App::new().service(get_radrad).service(radrad_play).service(radrad_stop))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}

