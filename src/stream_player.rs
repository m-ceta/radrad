use std::process::{Child, Command, Stdio};
use std::os::unix::io::{AsRawFd, FromRawFd};

pub fn play(url: &String, auth_token: &String) -> Result<(Child, Child), String> {
    let mut command1 = Command::new("ffmpeg");
    let args1 = ["-y", 
        "-headers", 
        &format!("\"X-Radiko-AuthToken: {}\"", auth_token),
        "-i",
        &format!("\"{}\"", url),
        "-"];
    let mut command2 = Command::new("ffplay");
    let args2 = ["-i", "-"];
    if let Ok(mut proc1) = command1.args(&args1).stdout(Stdio::piped()).spawn() {
        if let Ok(pipe) = stdout_to_stdin(&proc1) {
            if let Ok(proc2) = command2.args(&args2).stdout(pipe).spawn() {
                return Ok((proc1, proc2));
            } 
        }
        proc1.kill().unwrap_or("Failed to kill ffmpeg.");
        return Err(String::from("Failed to execute ffplay."));
    } 
    Err(String::from("Failed to execute ffmpeg."))
}

fn stdout_to_stdin(process: &Child) -> Option<Stdio> {
  if let Some(ref stdout) = process.stdout {
    return Some(unsafe { Stdio::from_raw_fd(stdout.as_raw_fd()) });
  }
  None
}

