use crate::config::Config;
use crate::error::{readable_thread_panic_error, SoundThreadError};
use crate::sound_bank::{SampleLoader, SoundBank};
use rodio::{OutputStream, Sink, OutputStreamHandle};
use std::fmt::Debug;
use std::sync::mpsc::{Sender, SendError, Receiver};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::thread;

#[derive(Debug)]
pub enum Operation {
    Stop,
    PlayBank {
        index: usize,
    },
}

struct BankState {
    sound_bank: SoundBank,
    sink: Option<Sink>,
}

struct SoundThreadBody {
    rx: Receiver<Operation>,
    banks: Vec<BankState>,
    // if _sound_output is dropped, sound_output_handle will no longer be usable
    _sound_output: OutputStream,
    sound_output_handle: OutputStreamHandle,
}

impl SoundThreadBody {
    fn new (config: Config, rx: Receiver<Operation>) -> Result<Self, SoundThreadError> {
        let mut loader = SampleLoader::new();
        loader.load_banks(&config.switches)?;
        let loader = loader;
        let banks = SoundBank::get_all(&loader, config.switches.clone())
            .expect("Failed to find sound sample, which should just have been loaded");

        let banks = banks.into_iter().map(|sound_bank| {
            BankState { sound_bank, sink: None }
        }).collect();

        // if _sound_output is dropped, sound_output_handle will no longer be usable
        let (_sound_output, sound_output_handle) = OutputStream::try_default()?;

        Ok(SoundThreadBody { rx, banks, _sound_output, sound_output_handle })
    }

    fn handle_operation_playbank(&mut self, index: usize,) -> Result<(), SoundThreadError> {
        if self.banks.get(index).is_none() {
            return Err(SoundThreadError::InvalidBankIndex);
        }

        let stop_all_sinks = self.banks[index].sound_bank.config.stop_sounds;

        if stop_all_sinks {
            // drop all sinks to stop all sounds
            for bank in self.banks.iter_mut() {
                bank.sink.take();
            }
        }

        let bank = &mut self.banks[index];

        if !stop_all_sinks {
            // Always drop the sink for the current bank. A dropped sink will stop playing. This prevents the same sound
            // from overlapping, or being queued up.
            bank.sink.take();
        }

        let sink = Sink::try_new(&self.sound_output_handle)?;
        bank.sound_bank.play(&sink)?;
        bank.sink = Some(sink);
        Ok(())
    }

    fn thread_body(mut self) -> Result<(), SoundThreadError> {
        loop {
            let received: Operation = self.rx.recv()?;

            // Should not return Err() from this point on, otherwise the whole thread stops
            // because of a single bad message (todo: consider making the communication duplex
            // so that we can return errors for individual operations)
            match received {
                Operation::Stop => {
                    return Ok(());
                }
                Operation::PlayBank { index } => {
                    if let Err(err) =  self.handle_operation_playbank(index) {
                        eprintln!("Failed to play sample: {:?}", err);
                    }
                }
            };
        }
    }
}

pub struct SoundThread {
    tx: Sender<Operation>,
    handle: JoinHandle<()>,
}

impl SoundThread {
    pub fn new(config: &Config) -> Result<Self, SoundThreadError> {
        let config = config.clone();
        let (tx, rx) = mpsc::channel();
        let (startup_tx, startup_rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            let result = SoundThreadBody::new(config, rx);
            match result {
                Ok(body) => {
                    startup_tx.send(Ok(()))
                        .expect("Failed to send SoundThread startup result to parent thread");

                    body.thread_body()
                        .expect("Error during SoundThreadBody.thread_body()");
                }
                Err(err) => {
                    startup_tx.send(Err(err))
                        .expect("Failed to send SoundThread startup result to parent thread");
                }
            }
        });

        startup_rx.recv()??;
        Ok(SoundThread { tx, handle })
    }

    pub fn stop(self) -> Result<(), SoundThreadError> {
        if let Err(err) = self.tx.send(Operation::Stop) {
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
    tx: Sender<Operation>,
}

impl SoundThreadRpc {
    pub fn new(thread: &SoundThread) -> Self {
        SoundThreadRpc { tx: thread.tx.clone() }
    }

    pub fn play(&self, index: usize) -> Result<(), SendError<Operation>> {
        self.tx.send(Operation::PlayBank { index })
    }
}
