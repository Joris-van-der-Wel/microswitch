use rodio::{StreamError, PlayError};
use rodio::decoder::DecoderError;
use std::any::Any;
use std::io;
use std::sync::mpsc::{RecvError, SendError, RecvTimeoutError};
use thiserror::Error;
use msgbox::IconType;
use std::path::PathBuf;
use std::fmt::{Debug, Display};
use crate::sound_thread::SoundThreadEvent;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {source}")]
    IO { #[from] source: io::Error },

    #[error("Failed to parse config file: {source}")]
    Parse { #[from] source: serde_yaml::Error },

    #[error("Unknown keyboard button \"{button}\". Allowed values are: {allowed_values}")]
    UnknownKeyboardButton { button: String, allowed_values: String },

    #[error("Unknown gamepad button \"{button}\". Allowed values are: {allowed_values}")]
    UnknownGamepadButton { button: String, allowed_values: String },

    #[error("Bank with id \"{bank}\" has not been defined")]
    UnknownBankId { bank: String },

    #[error("Sample with id \"{sample}\" has not been defined in bank \"{bank}\"")]
    UnknownSampleId { bank: String, sample: String },
}

#[derive(Error, Debug)]
pub enum SampleLoadError {
    #[error("Failed to read sample ({path}): {source}")]
    IO {
        path: PathBuf,
        source: io::Error,
    },

    #[error("Failed to decode sample ({path}): {source}")]
    Decode {
        path: PathBuf,
        source: DecoderError,
    },
}

#[derive(Error, Debug)]
#[error("Sample has not been loaded yet")]
pub struct SampleNotFoundError {}

#[derive(Error, Debug)]
pub enum SoundThreadError {
    #[error("SoundThread: Failed to read sample: {source}")]
    SampleLoad { #[from] source: SampleLoadError },

    #[error("SoundThread: Failed to open sound stream: {source}")]
    OpenSoundStream { #[from] source: StreamError },

    #[error("SoundThread: Failed to recv() from channel (sender went away?): {source}")]
    Recv { #[from] source: RecvError },

    #[error("SoundThread: Failed to send() SoundThreadEvent to channel: {source}")]
    SendEvent { #[from] source: SendError<SoundThreadEvent> },

    #[error("SoundThread: The thread panicked: {join_error_str}")]
    JoinPanic {
        join_error_str: String,
        join_error: Box<dyn Any + Send + 'static>,
    },

    #[error("SoundThread: Failed to play the sample")]
    PlayError { #[from] source: PlayError },
}

#[derive(Error, Debug)]
pub enum GamepadThreadError {
    #[error("GamepadThread: Failed to send message to SoundThread")]
    SendSoundThread,

    #[error("GamepadThread: Failed to recv() (sender went away?): {source}")]
    Recv { #[from] source: RecvError },

    #[error("GamepadThread: Failed to recv() (sender went away?): {source}")]
    Recv2 { #[from] source: RecvTimeoutError },

    #[error("GamepadThread: Failed to initialize gamepad library: {message}")]
    Gilrs { message: String },

    #[error("GamepadThread: The thread panicked: {join_error_str}")]
    JoinPanic {
        join_error_str: String,
        join_error: Box<dyn Any + Send + 'static>,
    },
}

// gilrs does not implement Send (on linux) which we need, so copy the error message only, instead
// of including it as `source`
impl From<gilrs::Error> for GamepadThreadError {
    fn from(error: gilrs::Error) -> Self {
        GamepadThreadError::Gilrs {
            message: format!("{}", error),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppRunError {
    #[error("Failed to start application (GamepadThread): {source}")]
    GamepadThread { #[from] source: GamepadThreadError },

    #[error("Failed to start application (SoundThread): {source}")]
    SoundThread { #[from] source: SoundThreadError },

    #[error("Failed to start application because the configuration is not valid: {source}")]
    Config { #[from] source: ConfigError },

    #[error("Failed to start application (iced): {source}")]
    Iced { #[from] source: iced::Error },
}

pub fn readable_thread_panic_error(error: &Box<dyn Any + Send + 'static>) -> String {
    let mut stringified = String::from("???");

    if let Some(s) = error.downcast_ref::<&str>() {
        stringified = format!("{}", s);
    }
    else if let Some(s) = error.downcast_ref::<String>() {
        stringified = format!("{}", s);
    }
    let type_id = error.type_id();

    format!("panic from thread: [{:?}]: [{}]", type_id, stringified)
}

pub fn error_msgbox<T: Display>(message: &'static str, error: &T) {
    let message = format!("{}: {}", message, error);
    println!("{}", &message);
    if let Err(err) = msgbox::create(concat!("μSwitch ", env!("CARGO_PKG_VERSION")), &message, IconType::Error) {
        println!("Failed to create msgbox: {:?}", err);
    }
}
