use crate::application::run_application;
use crate::config::Config;
use crate::gamepad_thread::GamepadThread;
use crate::sound_thread::{SoundThread, SoundThreadRpc};
use crate::error::AppRunError;
use path_absolutize::Absolutize;
use std::env;
use std::path::PathBuf;
use crate::sample_loader::{DiskSampleLoader, EmbeddedSampleLoader, SampleLoader};

pub mod config;
mod sound_bank;
mod application;
mod sound_thread;
mod gamepad_thread;
pub mod error;
mod sample_loader;

include! {
    // pub fn embedded_samples() -> HashMap<&'static str, &'static [u8]> { ... }
    // pub fn embedded_config() ->  Option<&'static str> { ... }
    concat!(env!("OUT_DIR"), "/embedded_config.rs")
}

pub fn run(mut args: env::Args) -> Result<(), AppRunError> {
    args.next().unwrap(); // skip executable path

    let (config, sample_loader): (Config, Box<dyn SampleLoader + Send>) = match embedded_config() {
        // no embedded config, try to read the config from a file
        None => {
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
            println!("Relative path for sample files is {}", config.resolve_path.as_ref().unwrap().to_str().unwrap());

            let sample_loader = Box::new(DiskSampleLoader::new());

            (config, sample_loader)
        },
        Some(embedded_config_yaml) => {
            println!("Using embedded config");
            let config = Config::from_string(embedded_config_yaml, None)?;
            let loader = EmbeddedSampleLoader::new(embedded_samples()).expect("Unable to load embedded sample");
            let sample_loader = Box::new(loader);

            (config, sample_loader)
        },
    };

    let (sound_thread, sound_thread_event_receiver) = SoundThread::new(&config, sample_loader)?;
    let gamepad_thread = GamepadThread::new(&config, SoundThreadRpc::new(&sound_thread))?;

    // this function will call process::exit() unless there was a startup error
    run_application(&config, sound_thread, gamepad_thread, sound_thread_event_receiver)?;

    panic!("This should have been unreachable");
}
