use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};
use std::path::PathBuf;
use std::sync::Arc;
use rodio::{Decoder, Sink, Source};
use rodio::source::{Buffered, SamplesConverter};
use crate::config::{BankConfig, SampleConfig};
use crate::error::SampleLoadError;

#[derive(Clone)]
enum LoadedSampleSource {
    Disk(Buffered<SamplesConverter<Decoder<BufReader<File>>, f32>>),
    Embedded(Buffered<SamplesConverter<Decoder<Cursor<&'static [u8]>>, f32>>),
}

fn decode<T: Read + Seek + Send>(data: T) -> Result<Buffered<SamplesConverter<Decoder<T>, f32>>, SampleLoadError> {
    let source = match Decoder::new(data) {
        Ok(v) => v,
        Err(err) => {
            return Err(SampleLoadError::Decode { source: err });
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

    Ok(source)
}

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
        match source {
            LoadedSampleSource::Disk(source) => sink.append(source),
            LoadedSampleSource::Embedded(source) => sink.append(source),
        };
    }
}

/// SampleLoader instances can be used to load audio samples from a source, in the form of a
/// LoadedSample. Every sample requested will be cached for the lifetime of the DiskSampleLoader
/// instance.
pub trait SampleLoader {
    /// Load and decode an audio sample, the source having been specified by the given SampleConfig.
    /// Subsequently, A reference to the sample can be obtained by passing the sample SampleConfig
    /// to the  get() method
    fn load_sample(&mut self, sample_config: &SampleConfig) -> Result<(), SampleLoadError>;

    /// Convenience method which calls load_sample for each SampleConfig specified by the given BankConfig
    fn load_bank(&mut self, bank_config: &BankConfig) -> Result<(), SampleLoadError> {
        for sample_config in &bank_config.samples {
            self.load_sample(sample_config)?;
        }

        Ok(())
    }

    /// Convenience method which calls load_bank for each BankConfig in the given list
    fn load_banks(&mut self, bank_configs: &Vec<BankConfig>) -> Result<(), SampleLoadError> {
        for bank_config in bank_configs.iter() {
            self.load_bank(bank_config)?;
        }

        Ok(())
    }

    /// Returns a reference to the appropriate LoadedSample for the given SampleConfig
    /// This sample must have been previously loaded by one of the load_x methods
    fn get(&self, sample_config: &SampleConfig) -> Option<Arc<LoadedSample>>;
}

/// A SampleLoader which reads samples from disk
pub struct DiskSampleLoader {
    loaded_samples: HashMap<PathBuf, Arc<LoadedSample>>,
}

impl DiskSampleLoader {
    pub fn new() -> Self {
        Self {
            loaded_samples: HashMap::new(),
        }
    }
}

impl SampleLoader for DiskSampleLoader {
    fn load_sample(&mut self, sample_config: &SampleConfig) -> Result<(), SampleLoadError> {
        let cache_key = sample_config.file.as_path();
        let path = sample_config.file_resolved.as_ref().unwrap().as_path();

        if !self.loaded_samples.contains_key(cache_key) {
            let file = match File::open(path) {
                Ok(v) => v,
                Err(err) => {
                    return Err(SampleLoadError::IO {source: err, path: path.to_path_buf()});
                }
            };
            let source = decode(BufReader::new(file))?;
            let loaded_sample = LoadedSample {
                source: LoadedSampleSource::Disk(source),
            };
            self.loaded_samples.insert(cache_key.to_path_buf(), Arc::new(loaded_sample));
        }

        Ok(())
    }

    fn get(&self, sample_config: &SampleConfig) -> Option<Arc<LoadedSample>> {
        let cache_key = sample_config.file.as_path();

        match self.loaded_samples.get(cache_key) {
            None => None,
            Some(sample) => Some(Arc::clone(sample)),
        }
    }
}

/// A SampleLoader which reads samples there were embedded in the binary during the build process
pub struct EmbeddedSampleLoader {
    loaded_samples: HashMap<PathBuf, Arc<LoadedSample>>,
}

impl EmbeddedSampleLoader {
    pub fn new(samples: HashMap<&'static str, &'static [u8]>) -> Result<Self, SampleLoadError> {
        let loaded_samples: Result<HashMap<PathBuf, Arc<LoadedSample>>, SampleLoadError> = samples.into_iter().map(|(key, data)| {
            let path = PathBuf::from(key);
            let source = decode(Cursor::new(data))?;
            let loaded_sample = LoadedSample {
                source: LoadedSampleSource::Embedded(source),
            };
            Ok((path, Arc::new(loaded_sample)))
        }).collect();

        Ok(Self {
            loaded_samples: loaded_samples?,
        })
    }
}

impl SampleLoader for EmbeddedSampleLoader {
    fn load_sample(&mut self, sample_config: &SampleConfig) -> Result<(), SampleLoadError> {
        let path = &sample_config.file;

        match self.loaded_samples.get(path) {
            None => {
                Err(SampleLoadError::EmbeddedSampleMissing { path: path.clone() })
            },
            Some(_) => Ok(()),
        }
    }

    fn get(&self, sample_config: &SampleConfig) -> Option<Arc<LoadedSample>> {
        let path = sample_config.file.as_path();

        match self.loaded_samples.get(path) {
            None => None,
            Some(sample) => Some(Arc::clone(sample)),
        }
    }
}
