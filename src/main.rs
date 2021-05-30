extern crate m3u8_rs;
extern crate eyre;
extern crate reqwest;
extern crate quick_xml;
extern crate regex;
extern crate base64;

use std::env;

mod rd_client;
mod stream_player;

#[cfg(target_os = "linux")]
extern crate signal_hook;
#[cfg(target_os = "linux")]
use signal_hook::consts::signal::*;
#[cfg(target_os = "linux")]
use std::thread;
#[cfg(target_os = "linux")]
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return;
    }
    let mut program = String::new();
    let mut tf = false;
    let mut af = false;
    for arg in args {
        if arg == "-tf" {
            tf = true;
        } else if arg == "-af" {
            af = true;
        } else if arg.len() > 0 {
            program = String::from(arg);
        }
    }
    if let Some((auth, url)) = rd_client::get_live_stream_info(&program, tf, af) {
        if let Ok(proc1) = stream_player::play(&url, &auth) {
            process_wait(proc1);
        } else {
            println!("Cannnot play the program '{}'.", program);
        }
    } else {
        println!("Not found the program '{}'.", program);
    }
}

#[cfg(target_os = "linux")]
fn process_wait(mut proc: std::process::Child) {
    let id = proc.id();
    let mut signals = signal_hook::iterator::Signals::new(&[SIGINT, SIGQUIT, SIGHUP, SIGTERM, SIGTSTP]).unwrap();
    thread::spawn(move || {
        let mut flag = true;
        for _sig in signals.forever() {
            if flag {
                Command::new("kill")
                    .arg(id.to_string())
                    .spawn()
                    .ok();
                flag = false;
            }
        }
    });
    proc.wait().ok();
}

#[cfg(target_os = "windows")]
fn process_wait(mut proc: std::process::Child) {
    proc.wait().ok();
}


