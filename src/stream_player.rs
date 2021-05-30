use std::process::{Child, Command, Stdio};

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

