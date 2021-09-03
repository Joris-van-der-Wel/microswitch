use crate::config::Config;
use crate::error::{readable_thread_panic_error, GamepadThreadError};
use crate::sound_thread::SoundThreadRpc;
use gilrs::{Gilrs, Event, EventType, Button, GamepadId};
use std::sync::mpsc::{Sender, RecvTimeoutError, Receiver};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub enum Operation {
    Stop,
}

struct GamepadThreadBody {
    config: Config,
    sound_thread_rpc: SoundThreadRpc,
    gilrs: Gilrs,
    rx: Receiver<Operation>,
}

impl GamepadThreadBody {
    fn new(config: Config, sound_thread_rpc: SoundThreadRpc, rx: Receiver<Operation>) -> Result<Self, GamepadThreadError> {
        let gilrs = Gilrs::new()?;
        let body = GamepadThreadBody { config, sound_thread_rpc, gilrs, rx };
        body.print_devices();
        Ok(body)
    }

    fn print_devices(&self) {
        println!("Connected gamepads:");
        for (id, gamepad) in self.gilrs.gamepads() {
            println!("  {}: \"{}\" is {:?}", id, gamepad.name(), gamepad.power_info());
        }
    }

    fn handle_gamepad_button(&self, gamepad_id: GamepadId, button: Button) -> Result<(), GamepadThreadError> {
        let device_id: usize = gamepad_id.into();
        println!("Gamepad {:?} button: {:?}", device_id, button);

        let switch_config = self.config.find_switch_for_gamepad_button(device_id, button);

        if let Some(switch_config) = switch_config {
            if let Err(err) = self.sound_thread_rpc.switch_pressed(switch_config.switch_ref) {
                eprintln!("Error sending play to sound thread {}", err);
                return Err(GamepadThreadError::SendSoundThread);
            }
        }

        Ok(())
    }

    fn thread_body(mut self) -> Result<(), GamepadThreadError> {
        let sleep_duration = Duration::from_millis(1);

        loop {
            while let Some(Event { id, event, time: _ }) = self.gilrs.next_event() {
                match event {
                    EventType::ButtonReleased(button, _code) => {
                        if let Err(err) = self.handle_gamepad_button(id, button) {
                            eprintln!("Error while handling gamepad button event {:?}", err);
                        }
                    },
                    _ => {},
                }
            }

            match self.rx.recv_timeout(sleep_duration) {
                Ok(received) => {
                    match received {
                        Operation::Stop => {
                            return Ok(());
                        }
                    }
                },
                Err(RecvTimeoutError::Timeout) => {
                    // continue the loop
                },
                Err(err @ RecvTimeoutError::Disconnected) => {
                    return Err(GamepadThreadError::from(err));
                },
            };
        }
    }
}

pub struct GamepadThread {
    tx: Sender<Operation>,
    handle: JoinHandle<()>,
}

impl GamepadThread {
    pub fn new(config: &Config, sound_thread_rpc: SoundThreadRpc) -> Result<Self, GamepadThreadError> {
        let config = config.clone();
        let (tx, rx) = mpsc::channel();
        let (startup_tx, startup_rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            let result = GamepadThreadBody::new(config, sound_thread_rpc, rx);
            match result {
                Ok(body) => {
                    startup_tx.send(Ok(()))
                        .expect("Failed to send GamepadThread startup result to parent thread");

                    body.thread_body()
                        .expect("Error during GamepadThreadBody.thread_body()");
                }
                Err(err) => {
                    startup_tx.send(Err(err))
                        .expect("Failed to send GamepadThread startup result to parent thread");
                }
            }
        });

        startup_rx.recv()??;
        Ok(GamepadThread { tx, handle })
    }

    pub fn stop(self) -> Result<(), GamepadThreadError> {
        if let Err(err) = self.tx.send(Operation::Stop) {
            eprintln!("Failed to send stop operation to SoundThread: {}", err);
            // Still try to join in this case, this will probably give us more error details
        }
        let handle = self.handle;

        match handle.join() {
            Ok(_) => Ok(()),
            Err(join_error) => {
                let join_error_str = readable_thread_panic_error(&join_error);
                Err(GamepadThreadError::JoinPanic {
                    join_error,
                    join_error_str,
                })
            }
        }
    }
}
