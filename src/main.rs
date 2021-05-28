extern crate m3u8_rs;
extern crate eyre;
extern crate reqwest;
extern crate quick_xml;
extern crate regex;
extern crate base64;
extern crate signal_hook;

use signal_hook::iterator::Signals;
use signal_hook::consts::signal::*;
use std::thread;
use std::process::Command;
use std::env;

mod rd_client;
mod stream_player;

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
    if let Some((auth, url)) = rd_client::get_live_stream_info(program, tf, af) {
        if let Ok((mut proc1, mut proc2)) = stream_player::play(&url, &auth) {
            let mut signals = Signals::new(&[SIGINT, SIGHUP, SIGQUIT, SIGKILL, SIGTERM]).unwrap();
            let id1 = proc1.id();
            let id2 = proc2.id();
            thread::spawn(move || {
                let mut flag = true;
                for _sig in signals.forever() {
                    if flag {
                        Command::new("kill").arg("-9").arg(id1.to_string()).spawn().unwrap();
                        Command::new("kill").arg("-9").arg(id2.to_string()).spawn().unwrap();
                        flag = false;
                    }
                }
            });
            proc1.wait().unwrap();
            proc2.wait().unwrap();
        }
    } else {
        println!("Not found program '{}'.", program);
    }
}

