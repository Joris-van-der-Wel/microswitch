use crate::config::{Config, SwitchRef, BankSampleRef};
use crate::error::{readable_thread_panic_error, SoundThreadError};
use crate::sound_bank::{SampleLoader, SoundBank, SoundBankState};
use rodio::{OutputStream};
use std::fmt::Debug;
use std::sync::mpsc::{Sender, SendError, Receiver};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::thread;

#[derive(Debug)]
pub enum Operation {
    Stop,
    SwitchPressed {
        switch_ref: SwitchRef,
    },
}

#[derive(Debug)]
pub enum SoundThreadEvent {
    PlayedSample(BankSampleRef),
}

/// A single SoundThreadBody instance is created for each spawned sound thread, in order to track
/// the state of the track.
struct SoundThreadBody {
    operation_receiver: Receiver<Operation>,
    event_sender: Sender<SoundThreadEvent>,
    config: Config,
    banks: Vec<SoundBankState>,
    // if _sound_output is dropped, sound_output_handle will no longer be usable
    _sound_output: OutputStream,
}

impl SoundThreadBody {
    fn new (config: Config, operation_receiver: Receiver<Operation>, event_sender: Sender<SoundThreadEvent>) -> Result<Self, SoundThreadError> {
        let mut loader = SampleLoader::new();
        loader.load_banks(&config.banks)?;
        let loader = loader;

        let banks = SoundBank::new_all(&loader, config.banks.clone())
            .expect("SoundThread: Failed to find sound sample, which should just have been loaded");

        // if _sound_output is dropped, sound_output_handle will no longer be usable
        let (_sound_output, sound_output_handle) = OutputStream::try_default()?;

        let banks = SoundBankState::new_all(&sound_output_handle, banks);

        Ok(Self { operation_receiver, event_sender, config, banks, _sound_output })
    }

    fn handle_operation_switch_pressed(&mut self, switch_ref: SwitchRef) -> Result<(), SoundThreadError> {
        let switch_config = self.config.switch(switch_ref);

        if switch_config.stop_sounds {
            for bank in self.banks.iter_mut() {
                bank.stop();
            }
        }

        if let Some(play) = &switch_config.play {
            let bank_sample_ref = play.bank_sample_ref;
            let bank_state = &mut self.banks[bank_sample_ref.bank.bank_index];
            bank_state.play(bank_sample_ref.sample)?;
            self.event_sender.send(SoundThreadEvent::PlayedSample(bank_sample_ref))?;
        }

        if let Some(play) = &switch_config.play_random {
            let bank_ref = play.bank_ref;
            let bank_state = &mut self.banks[bank_ref.bank_index];
            if let Some(bank_sample_ref) = bank_state.play_random()? {
                self.event_sender.send(SoundThreadEvent::PlayedSample(bank_sample_ref))?;
            }
        }

        if let Some(play) = &switch_config.play_step {
            let bank_state = &mut self.banks[play.bank_ref.bank_index];
            if let Some(bank_sample_ref) = bank_state.play_step(play.steps)? {
                self.event_sender.send(SoundThreadEvent::PlayedSample(bank_sample_ref))?;
            }
        }

        Ok(())
    }

    fn thread_body(mut self) -> Result<(), SoundThreadError> {
        loop {
            let received: Operation = self.operation_receiver.recv()?;

            // Should not return Err() from this point on, otherwise the whole thread stops
            // because of a single bad message (todo: consider making the communication duplex
            // so that we can return errors for individual operations)
            match received {
                Operation::Stop => {
                    return Ok(());
                }
                Operation::SwitchPressed { switch_ref } => {
                    if let Err(err) =  self.handle_operation_switch_pressed(switch_ref) {
                        eprintln!("SoundThread: Failed to handle switch press: {:?}", err);
                    }
                }
            };
        }
    }
}

/// The SoundThread takes care of loading audio samples from disk, playing them using rodio and
/// tracking the play state of banks/samples.
/// A SoundThread instance is created by the parent process in order to spawn the actual thread and
/// has methods for communicating with the thread.
pub struct SoundThread {
    operation_sender: Sender<Operation>,
    handle: JoinHandle<()>,
}

impl SoundThread {
    pub fn new(config: &Config) -> Result<(Self, Receiver<SoundThreadEvent>), SoundThreadError> {
        let config = config.clone();
        let (operation_sender, operation_receiver) = mpsc::channel();
        let (startup_sender, startup_receiver) = mpsc::channel();
        let (event_sender, event_receiver) = mpsc::channel();

        let handle = thread::spawn(move || {
            let result = SoundThreadBody::new(config, operation_receiver, event_sender);
            match result {
                Ok(body) => {
                    startup_sender.send(Ok(()))
                        .expect("SoundThread: Failed to send startup result to parent thread");

                    body.thread_body()
                        .expect("SoundThread: Error during SoundThreadBody.thread_body()");
                }
                Err(err) => {
                    startup_sender.send(Err(err))
                        .expect("SoundThread: Failed to send startup result to parent thread");
                }
            }
        });

        startup_receiver.recv()??;
        let sound_thread = SoundThread { operation_sender, handle };
        Ok((sound_thread, event_receiver))
    }

    pub fn stop(self) -> Result<(), SoundThreadError> {
        if let Err(err) = self.operation_sender.send(Operation::Stop) {
            eprintln!("Failed to send stop operation to SoundThread: {}", err);
            // Still try to join in this case, this will probably give us more error details
        }
        let handle = self.handle;

        match handle.join() {
            Ok(_) => Ok(()),
            Err(join_error) => {
                let join_error_str = readable_thread_panic_error(&join_error);
                Err(SoundThreadError::JoinPanic {
                    join_error,
                    join_error_str,
                })
            }
        }
    }
}

pub struct SoundThreadRpc {
    operation_sender: Sender<Operation>,
}

impl SoundThreadRpc {
    pub fn new(thread: &SoundThread) -> Self {
        SoundThreadRpc { operation_sender: thread.operation_sender.clone() }
    }

    pub fn switch_pressed(&self, switch_ref: SwitchRef) -> Result<(), SendError<Operation>> {
        self.operation_sender.send(Operation::SwitchPressed { switch_ref })
    }
}
