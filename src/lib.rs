extern crate regex;
extern crate glob;
extern crate crossterm;
use std::process::Command;
use std::error::Error;
use std::env;
use regex::Regex;
use glob::glob;

fn find_lib(lib: String) -> Result<String, Box<dyn Error>> {
    let output = Command::new("ldconfig").arg("-p").output()?;
    let pattern = Regex::new(&format!(r"^[\s]*lib{}.*.so .* ([^ ]+)$", lib))?;
    let list = String::from_utf8(output.stdout)?
        .lines()
        .filter_map(|line| pattern.captures(line) )
        .map(|cap| cap[1].to_string())
        .collect::<Vec<String>>();
    if list.is_empty() {
        Err(std::boxed::Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("failed to load lib {}", lib))))
    }
    else {
        Ok(list[0].to_string())
    }
}

pub fn ld_preload_path(player: &String) -> Result<String, Box<dyn Error>> {
    let exe = env::current_exe()?;
    let bin_dir = exe.parent().ok_or("bin_dir")?.to_str().ok_or("to str")?;
    let mut ld_preload : String = "".to_string();
    let mut found = false;
    let pattern = bin_dir.to_string() + "/**/libblockish_caca*.so";
    for entry in glob(&(pattern))? {
        if let Ok(path) = entry {
            if let Some(str_path) = path.as_path().to_str() {
                ld_preload += &(":".to_string() + str_path);
                found = true;
            }
        }
    }
    if !found {
        ld_preload += &find_lib("blockish_caca".to_string())?;
    }
    if player == "cvlc" {
        ld_preload += &":".to_string();
        ld_preload += &find_lib("caca".to_string())?;
    }
    Ok(ld_preload)
}


pub fn video_command(player: &String, path: &String) -> Result<Command, Box<dyn Error>> {
    let mut quiet = "-quiet";
    let mut vo = "-vo";
    let mut com = Command::new(player);
    let ld_preload = ld_preload_path(&player)?;
    let mut bwidth = 80;
    let mut bheight = 20;
    if let Ok(res) = crossterm::terminal::size() {
        bwidth = res.0;
        bheight = res.1 
    }
    if player == "cvlc" {
        quiet = "--quiet";
        vo = "-V";
        com.env("DISPLAY", "");
    }
    com
        .env("COLUNMS", bwidth.to_string())
        .env("LINES", bheight.to_string())
        .env("CACA_DRIVER", "raw")
        .env("LD_PRELOAD", ld_preload)
        .arg(quiet)
        .arg(vo)
        .arg("caca")
        .arg(path);
    Ok(com)
}

#[cfg(target_os="unix")]
pub fn play_video(player: &String, path: &String) -> Result<(), Box<dyn Error>> {
    use std::os::unix::process::CommandExt;
    video_command(player, path)?.exec();
    Ok(())
}
