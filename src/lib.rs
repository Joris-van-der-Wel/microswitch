use crate::application::run_application;
use crate::config::Config;
use crate::gamepad_thread::GamepadThread;
use crate::sound_thread::{SoundThread, SoundThreadRpc};
use crate::error::AppRunError;
use path_absolutize::Absolutize;
use std::env;
use std::path::PathBuf;


pub mod config;
mod sound_bank;
mod application;
mod sound_thread;
mod gamepad_thread;
pub mod error;

pub fn run(mut args: env::Args) -> Result<(), AppRunError> {
    args.next().unwrap(); // skip executable path

    let config_path: PathBuf = if args.len() > 0 {
        PathBuf::from(&args.next().unwrap())
    } else {
        let mut exe_path = std::env::current_exe().unwrap();
        exe_path.pop();
        exe_path.push("config");
        exe_path.push("config.yaml");
        exe_path
    };

    let config_path = config_path.absolutize().unwrap().into_owned();
    let config_path = config_path.as_path();

    println!("Using config file: {}", config_path.to_str().unwrap());
    let config = Config::from_file(config_path)?;
    println!("Relative path for sample files is {}", config.resolve_path.to_str().unwrap());

    let sound_thread = SoundThread::new(&config)?;
    let gamepad_thread = GamepadThread::new(&config, SoundThreadRpc::new(&sound_thread))?;

    // this function will call process::exit() unless there was a startup error
    run_application(&config, sound_thread, gamepad_thread)?;

    panic!("This should have been unreachable");
}
