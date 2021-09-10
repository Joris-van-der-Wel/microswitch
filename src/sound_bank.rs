use crate::config::{BankConfig, SampleConfig, SampleRef, BankSampleRef};
use crate::error::{SampleNotFoundError, SampleLoadError};
use rand::{thread_rng, Rng};
use rodio::source::{Buffered, SamplesConverter};
use rodio::{Decoder, PlayError, Sink, Source, OutputStreamHandle};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::rc::Rc;
use std::convert::TryFrom;

type LoadedSampleSource = Buffered<SamplesConverter<Decoder<BufReader<File>>, f32>>;

/// LoadedSample represents a single sound sample loaded and decoded into memory
pub struct LoadedSample {
    source: LoadedSampleSource,
}

impl LoadedSample {
    /// Returns a source in rodio's expected format.
    fn to_source(&self) -> LoadedSampleSource {
        self.source.clone()
    }

    /// Plays the sampe using the given rodio Sink
    pub fn play(&self, sink: &Sink) {
        let source = self.to_source();
        sink.append(source);
    }
}

/// SampleLoader instances can be used to load audio samples from disk, in the form of a
/// LoadedSample. Every sample requested will be cached for the lifetime of the SampleLoader
/// instance.
pub struct SampleLoader {
    loaded_samples: HashMap<PathBuf, Rc<LoadedSample>>,
}

impl SampleLoader {
    pub fn new() -> Self {
        SampleLoader {
            loaded_samples: HashMap::new(),
        }
    }

    /// Load and decode an audio sample from disk at the path specified by the given SampleConfig.
    /// Subsequently, A reference to the sample can be obtained by passing the sample SampleConfig
    /// to the  get() method
    pub fn load_sample(&mut self, sample_config: &SampleConfig) -> Result<(), SampleLoadError> {
        let path = sample_config.file_resolved.as_path();

        if !self.loaded_samples.contains_key(path) {
            let file = match File::open(path) {
                Ok(v) => v,
                Err(err) => {
                    return Err(SampleLoadError::IO {source: err, path: path.to_path_buf()});
                }
            };
            let source = match Decoder::new(BufReader::new(file)) {
                Ok(v) => v,
                Err(err) => {
                    return Err(SampleLoadError::Decode {source: err, path: path.to_path_buf()});
                }
            };
            let source = source.convert_samples::<f32>();
            let source = source.buffered();

            // Fill the internal buffer of Buffered, later we can use .clone() to get a copy with all
            // data already decoded.
            {
                let mut source_clone = source.clone();
                loop {
                    if let None = source_clone.next() {
                        break;
                    }
                }
            }

            let loaded_sample = LoadedSample {
                source,
            };

            self.loaded_samples.insert(path.to_path_buf(), Rc::new(loaded_sample));
        }

        Ok(())
    }

    /// Convenience method which calls load_sample for each SampleConfig specified by the given BankConfig
    pub fn load_bank(&mut self, bank_config: &BankConfig) -> Result<(), SampleLoadError> {
        for sample_config in &bank_config.samples {
            self.load_sample(sample_config)?;
        }

        Ok(())
    }

    /// Convenience method which calls load_bank for each BankConfig in the given list
    pub fn load_banks(&mut self, bank_configs: &Vec<BankConfig>) -> Result<(), SampleLoadError> {
        for bank_config in bank_configs.iter() {
            self.load_bank(bank_config)?;
        }

        Ok(())
    }

    /// Returns a reference to the appropriate LoadedSample for the given SampleConfig
    /// This sample must have been previously loaded by one of the load_x methods
    pub fn get(&self, sample_config: &SampleConfig) -> Option<Rc<LoadedSample>> {
        match self.loaded_samples.get(&sample_config.file_resolved) {
            None => None,
            Some(sample) => Some(Rc::clone(sample)),
        }
    }
}

/// An initialized bank of sounds. This is the combination of a BankConfig and all of its audio
/// samples loaded into memory.
pub struct SoundBank {
    pub config: BankConfig,
    samples: Vec<Rc<LoadedSample>>,
}

impl SoundBank {
    /// Create a new SoundBank by parsing the config and loading all audio samples into memory.
    pub fn new(loader: &SampleLoader, config: BankConfig) -> Result<SoundBank, SampleNotFoundError> {
        let mut samples: Vec<Rc<LoadedSample>> = Vec::new();

        for sample_config in &config.samples {
            let sample = match loader.get(sample_config) {
                None => {
                    return Err(SampleNotFoundError {});
                },
                Some(v) => v,
            };
            samples.push(sample);
        }

        Ok(SoundBank { config, samples })
    }

    /// Convenience method which calls and returns new() for each BankConfig in the given list
    pub fn new_all(loader: &SampleLoader, configs: Vec<BankConfig>) -> Result<Vec<SoundBank>, SampleNotFoundError> {
        let banks: Result<Vec<_>, _> = configs
            .into_iter()
            .map(|config| { SoundBank::new(loader, config) })
            .collect();
        banks
    }

    /// The amount of samples specified by this bank
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Returns the appropriate LoadedSample based on a SampleRef
    pub fn get_sample(&self, sample_ref: SampleRef) -> Rc<LoadedSample> {
        Rc::clone(&self.samples[sample_ref.sample_index])
    }
}

fn apply_steps(sample_count: usize, last_played: Option<SampleRef>, steps: i32) -> Option<SampleRef> {
    if sample_count < 1 {
        return None;
    }

    match last_played {
        // This bank has not been played before:
        None => {
            if steps >= 0 {
                Some(SampleRef { sample_index: 0 })
            }
            else { // < 0
                Some(SampleRef { sample_index: sample_count - 1 })
            }
        },

        Some(last_played) => {
            let max = i32::try_from(sample_count).unwrap_or(0);
            let last_index = i32::try_from(last_played.sample_index).unwrap_or(0);

            let next = (((last_index + steps) % max) + max) % max;
            let next = usize::try_from(next).unwrap_or(0);
            Some(SampleRef { sample_index: next })
        },
    }
}

/// An initialized bank of sounds and its runtime state. This is the combination of a SoundBank,
/// rodio objects used for playback, and the state needed to properly apply SwitchPlayStep actions.
pub struct SoundBankState {
    sound_output_handle: OutputStreamHandle,
    sound_bank: SoundBank,
    // A sink is Option so that we can destroy individual sinks
    sinks: Vec<Option<Sink>>,
    last_played: Option<SampleRef>,
}

impl SoundBankState {
    pub fn new(sound_output_handle: OutputStreamHandle, sound_bank: SoundBank) -> Self {
        // reserve a location in a vector of sinks, for every sample, but set them to None for now.
        let sinks = sound_bank.samples.iter().map(|_sample| None).collect();

        SoundBankState {
            sound_output_handle,
            sound_bank,
            sinks,
            last_played: None,
        }
    }

    pub fn new_all(sound_output_handle: &OutputStreamHandle, sound_banks: Vec<SoundBank>) -> Vec<Self> {
        sound_banks
            .into_iter()
            .map(|sound_bank| SoundBankState::new(sound_output_handle.clone(), sound_bank))
            .collect()
    }

    fn stop_if_not_poly(&mut self) {
        if !self.sound_bank.config.poly {
            self.stop();
        }
    }


    fn pick_random_sample(&self) -> Option<SampleRef> {
        let sample_count = self.sound_bank.sample_count();

        if sample_count < 1 {
            None
        }
        else {
            let mut rng = thread_rng();
            let sample_index = rng.gen_range(0..sample_count);
            Some(SampleRef { sample_index })
        }
    }

    pub fn stop(&mut self) {
        // drop all sinks to stop all sounds
        for sink in self.sinks.iter_mut() {
            sink.take();
        }
    }

    pub fn play(&mut self, sample_ref: SampleRef) -> Result<BankSampleRef, PlayError> {
        self.stop_if_not_poly();
        self.last_played = Some(sample_ref);
        let sample = self.sound_bank.get_sample(sample_ref);

        let sink = Sink::try_new(&self.sound_output_handle)?;
        self.sinks[sample_ref.sample_index] = Some(sink);
        let sink = self.sinks[sample_ref.sample_index].as_ref().unwrap();

        let bank_config = &self.sound_bank.config;
        let sample_config = bank_config.sample(sample_ref);
        println!("Playing from bank \"{}\", the sample \"{}\"", bank_config.id.as_str(), sample_config.id.as_str());

        sample.play(sink);
        Ok(BankSampleRef {
            bank: self.sound_bank.config.bank_ref,
            sample: sample_ref,
        })
    }

    pub fn play_random(&mut self) -> Result<Option<BankSampleRef>, PlayError> {
        match self.pick_random_sample() {
            None => Ok(None),
            Some(sample_ref) => Ok(Some(self.play(sample_ref)?)),
        }
    }

    pub fn play_step(&mut self, steps: i32)  -> Result<Option<BankSampleRef>, PlayError> {
        if let Some(sample_ref) = apply_steps(self.sound_bank.sample_count(), self.last_played, steps) {
            Ok(Some(self.play(sample_ref)?))
        }
        else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sound_bank::apply_steps;
    use crate::config::SampleRef;

    #[test]
    fn apply_steps_calculation() {
        let s = |sample_index| Some(SampleRef { sample_index });

        // Should start with a default sample if the bank has not been played before
        assert_eq!(apply_steps(4, None, -1), s(3));
        assert_eq!(apply_steps(4, None, 0), s(0));
        assert_eq!(apply_steps(4, None, 1), s(0));

        // Should repeat the previous sample if steps is 0
        assert_eq!(apply_steps(4, s(0), 0), s(0));
        assert_eq!(apply_steps(4, s(1), 0), s(1));
        assert_eq!(apply_steps(4, s(3), 0), s(3));

        // Should go backwards, with wraparound, if steps < 0
        assert_eq!(apply_steps(4, s(0), -1), s(3));
        assert_eq!(apply_steps(4, s(1), -1), s(0));
        assert_eq!(apply_steps(4, s(2), -2), s(0));
        assert_eq!(apply_steps(4, s(1), -2), s(3));
        assert_eq!(apply_steps(4, s(3), -5), s(2));

        // Should go forwards, with wraparound, if steps > 0
        assert_eq!(apply_steps(4, s(0), 1), s(1));
        assert_eq!(apply_steps(4, s(3), 1), s(0));
        assert_eq!(apply_steps(4, s(1), 2), s(3));
        assert_eq!(apply_steps(4, s(3), 2), s(1));
        assert_eq!(apply_steps(4, s(2), 5), s(3));
    }
}
