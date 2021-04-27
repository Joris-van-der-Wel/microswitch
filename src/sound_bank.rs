use crate::config::{SwitchConfig, SampleConfig};
use crate::error::{SampleNotFoundError, SampleLoadError};
use rand::{thread_rng, Rng};
use rodio::source::{Buffered, SamplesConverter};
use rodio::{Decoder, PlayError, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::rc::Rc;

type LoadedSampleSource = Buffered<SamplesConverter<Decoder<BufReader<File>>, f32>>;

pub struct LoadedSample {
    original_path: PathBuf,
    source: LoadedSampleSource,
}

impl LoadedSample {
    fn to_source(&self) -> LoadedSampleSource {
        self.source.clone()
    }
}

pub struct SampleLoader {
    loaded_samples: HashMap<PathBuf, Rc<LoadedSample>>,
}

impl SampleLoader {
    pub fn new() -> Self {
        SampleLoader {
            loaded_samples: HashMap::new(),
        }
    }

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
                original_path: path.to_path_buf(),
                source,
            };

            self.loaded_samples.insert(path.to_path_buf(), Rc::new(loaded_sample));
        }

        Ok(())
    }

    pub fn load_bank(&mut self, switch_config: &SwitchConfig) -> Result<(), SampleLoadError> {
        for sample_config in &switch_config.samples {
            self.load_sample(sample_config)?;
        }

        Ok(())
    }

    pub fn load_banks(&mut self, switch_configs: &Vec<SwitchConfig>) -> Result<(), SampleLoadError> {
        for bank_config in switch_configs.iter() {
            self.load_bank(bank_config)?;
        }

        Ok(())
    }

    pub fn get(&self, sample_config: &SampleConfig) -> Option<Rc<LoadedSample>> {
        match self.loaded_samples.get(&sample_config.file_resolved) {
            None => None,
            Some(sample) => Some(Rc::clone(sample)),
        }
    }
}

pub struct SoundBank {
    pub config: SwitchConfig,
    samples: Vec<Rc<LoadedSample>>,
}

impl SoundBank {
    pub fn get(loader: &SampleLoader, config: SwitchConfig) -> Result<SoundBank, SampleNotFoundError> {
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

    pub fn get_all(loader: &SampleLoader, configs: Vec<SwitchConfig>) -> Result<Vec<SoundBank>, SampleNotFoundError> {
        let banks: Result<Vec<_>, _> = configs
            .into_iter()
            .map(|config| { SoundBank::get(loader, config) })
            .collect();
        banks
    }

    fn pick_random_sample(&self) -> Option<Rc<LoadedSample>> {
        if self.samples.is_empty() {
            None
        }
        else {
            let mut rng = thread_rng();
            let index = rng.gen_range(0..self.samples.len());
            Some(Rc::clone(&self.samples[index]))
        }
    }

    pub fn play(&self, sink: &Sink) -> Result<(), PlayError> {
        let sample = match self.pick_random_sample() {
            None => { return Ok(()); } // there are no samples
            Some(v) => v
        };

        println!("Playing from bank {}, the sample {}", self.config.title, sample.original_path.to_str().unwrap());

        let source = sample.to_source();
        sink.append(source);

        Ok(())
    }
}
