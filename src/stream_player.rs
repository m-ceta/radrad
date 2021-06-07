use std::process::{Child, Command, Stdio};

pub struct Player {
    station_id: Option<String>,
    option_af: bool,
    option_tf: bool,
    command: Option<Child>,
}

impl Player {

    pub fn new() -> Player {
        Player {
            station_id: None,
            option_af: false,
            option_tf: false,
            command: None,
        }
    }

    pub fn play(self: &mut Player, station_id: &String, option_af: bool, option_tf: bool) {
        self.stop();
        if let Some((auth, url)) = super::rd_client::get_live_stream_info(&station_id, option_tf, option_af) {
            if let Ok(proc) = play(&url, &auth) {
                self.station_id = Some(station_id.clone());
                self.option_af = option_af;
                self.option_tf = option_tf;
                self.command = Some(proc);
            }
        }
    }

    pub fn stop(self: &mut Player) {
        if let Some(ref mut cmd) = self.command {
            cmd.kill().unwrap_or(());
        }
        self.station_id = None;
        self.option_af = false;
        self.option_tf = false;
        self.command = None;
    }

    pub fn get_station_id(self: &Player) -> Option<String> {
        if let Some(ref stat_id) = &self.station_id {
            return Some(stat_id.clone());
        }
        None
    }

    pub fn get_option_af(self: &Player) -> bool {
        self.option_af
    }

    pub fn get_option_tf(self: &Player) -> bool {
        self.option_tf
    }

    pub fn get_child_id(self: &Player) -> Option<u32> {
        if let Some(ref child) = &self.command {
            return Some(child.id());
        }
        None
    }
}

pub fn play(url: &String, auth_token: &String) -> Result<Child, String> {
    let mut command1 = Command::new("ffplay");
    let args1 = ["-headers"
        , &format!("X-Radiko-AuthToken:{}", auth_token)
        , "-i"
        , &format!("{}", url)
        , "-nodisp"];
    if let Ok(proc1) = command1.args(&args1).stdout(Stdio::piped()).spawn() {
        return Ok(proc1);
    } 
    Err(String::from("Failed to execute ffmpeg."))
}

