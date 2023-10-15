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
enum GamepadOperation {
    Stop,
}

struct GamepadThreadBody {
    config: Config,
    sound_thread_rpc: SoundThreadRpc,
    gilrs: Gilrs,
    operation_receiver: Receiver<GamepadOperation>,
}

impl GamepadThreadBody {
    fn new(config: Config, sound_thread_rpc: SoundThreadRpc, operation_receiver: Receiver<GamepadOperation>) -> Result<Self, GamepadThreadError> {
        let gilrs = Gilrs::new()?;
        let body = Self { config, sound_thread_rpc, gilrs, operation_receiver };
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
            let switch_ref = switch_config.switch_ref;

            if let Err(err) = self.sound_thread_rpc.switch_pressed(switch_ref) {
                eprintln!("Error sending switch_pressed {:?} to sound thread {}", switch_ref, err);
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
                    EventType::ButtonPressed(button, _code) => {
                        if let Err(err) = self.handle_gamepad_button(id, button) {
                            eprintln!("Error while handling gamepad button event {:?}", err);
                        }
                    },
                    _ => {},
                }
            }

            match self.operation_receiver.recv_timeout(sleep_duration) {
                Ok(received) => {
                    match received {
                        GamepadOperation::Stop => {
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
    operation_sender: Sender<GamepadOperation>,
    handle: JoinHandle<()>,
}

impl GamepadThread {
    pub fn new(
        config: &Config,
        sound_thread_rpc: SoundThreadRpc,
    ) -> Result<Self, GamepadThreadError> {
        let config = config.clone();
        let (operation_sender, operation_receiver) = mpsc::channel();
        let (startup_sender, startup_receiver) = mpsc::channel();

        let handle = thread::spawn(move || {
            let result = GamepadThreadBody::new(config, sound_thread_rpc, operation_receiver);
            match result {
                Ok(body) => {
                    startup_sender.send(Ok(()))
                        .expect("Failed to send GamepadThread startup result to parent thread");

                    body.thread_body()
                        .expect("Error during GamepadThreadBody.thread_body()");
                }
                Err(err) => {
                    startup_sender.send(Err(err))
                        .expect("Failed to send GamepadThread startup result to parent thread");
                }
            }
        });

        startup_receiver.recv()??;
        let gamepad_thread = Self { operation_sender, handle };
        Ok(gamepad_thread)
    }

    pub fn stop(self) -> Result<(), GamepadThreadError> {
        if let Err(err) = self.operation_sender.send(GamepadOperation::Stop) {
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
