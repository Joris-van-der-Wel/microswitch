use crate::error::ConfigError;
use gilrs::Button;
use iced::keyboard::KeyCode;
use serde::{Deserialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::{fs};

const KEYBOARD_BUTTON_MAPPING: [(&str, KeyCode); 136] = [
    ("1", KeyCode::Key1),
    ("2", KeyCode::Key2),
    ("3", KeyCode::Key3),
    ("4", KeyCode::Key4),
    ("5", KeyCode::Key5),
    ("6", KeyCode::Key6),
    ("7", KeyCode::Key7),
    ("8", KeyCode::Key8),
    ("9", KeyCode::Key9),
    ("0", KeyCode::Key0),
    ("A", KeyCode::A),
    ("B", KeyCode::B),
    ("C", KeyCode::C),
    ("D", KeyCode::D),
    ("E", KeyCode::E),
    ("F", KeyCode::F),
    ("G", KeyCode::G),
    ("H", KeyCode::H),
    ("I", KeyCode::I),
    ("J", KeyCode::J),
    ("K", KeyCode::K),
    ("L", KeyCode::L),
    ("M", KeyCode::M),
    ("N", KeyCode::N),
    ("O", KeyCode::O),
    ("P", KeyCode::P),
    ("Q", KeyCode::Q),
    ("R", KeyCode::R),
    ("S", KeyCode::S),
    ("T", KeyCode::T),
    ("U", KeyCode::U),
    ("V", KeyCode::V),
    ("W", KeyCode::W),
    ("X", KeyCode::X),
    ("Y", KeyCode::Y),
    ("Z", KeyCode::Z),
    ("Escape", KeyCode::Escape),
    ("F1", KeyCode::F1),
    ("F2", KeyCode::F2),
    ("F3", KeyCode::F3),
    ("F4", KeyCode::F4),
    ("F5", KeyCode::F5),
    ("F6", KeyCode::F6),
    ("F7", KeyCode::F7),
    ("F8", KeyCode::F8),
    ("F9", KeyCode::F9),
    ("F10", KeyCode::F10),
    ("F11", KeyCode::F11),
    ("F12", KeyCode::F12),
    ("F13", KeyCode::F13),
    ("F14", KeyCode::F14),
    ("F15", KeyCode::F15),
    ("F16", KeyCode::F16),
    ("F17", KeyCode::F17),
    ("F18", KeyCode::F18),
    ("F19", KeyCode::F19),
    ("F20", KeyCode::F20),
    ("F21", KeyCode::F21),
    ("F22", KeyCode::F22),
    ("F23", KeyCode::F23),
    ("F24", KeyCode::F24),
    ("Scroll", KeyCode::Scroll),
    ("Pause", KeyCode::Pause),
    ("Insert", KeyCode::Insert),
    ("Home", KeyCode::Home),
    ("Delete", KeyCode::Delete),
    ("End", KeyCode::End),
    ("PageDown", KeyCode::PageDown),
    ("PageUp", KeyCode::PageUp),
    ("Left", KeyCode::Left),
    ("Up", KeyCode::Up),
    ("Right", KeyCode::Right),
    ("Down", KeyCode::Down),
    ("Backspace", KeyCode::Backspace),
    ("Enter", KeyCode::Enter),
    ("Space", KeyCode::Space),
    ("Compose", KeyCode::Compose),
    ("Caret", KeyCode::Caret),
    ("Numlock", KeyCode::Numlock),
    ("Numpad0", KeyCode::Numpad0),
    ("Numpad1", KeyCode::Numpad1),
    ("Numpad2", KeyCode::Numpad2),
    ("Numpad3", KeyCode::Numpad3),
    ("Numpad4", KeyCode::Numpad4),
    ("Numpad5", KeyCode::Numpad5),
    ("Numpad6", KeyCode::Numpad6),
    ("Numpad7", KeyCode::Numpad7),
    ("Numpad8", KeyCode::Numpad8),
    ("Numpad9", KeyCode::Numpad9),
    ("NumpadAdd", KeyCode::NumpadAdd),
    ("NumpadDivide", KeyCode::NumpadDivide),
    ("NumpadDecimal", KeyCode::NumpadDecimal),
    ("NumpadComma", KeyCode::NumpadComma),
    ("NumpadEnter", KeyCode::NumpadEnter),
    ("NumpadEquals", KeyCode::NumpadEquals),
    ("NumpadMultiply", KeyCode::NumpadMultiply),
    ("NumpadSubtract", KeyCode::NumpadSubtract),
    ("Apostrophe", KeyCode::Apostrophe),
    ("Asterisk", KeyCode::Asterisk),
    ("Backslash", KeyCode::Backslash),
    ("Capital", KeyCode::Capital),
    ("Colon", KeyCode::Colon),
    ("Comma", KeyCode::Comma),
    ("Convert", KeyCode::Convert),
    ("Equals", KeyCode::Equals),
    ("Grave", KeyCode::Grave),
    ("LAlt", KeyCode::LAlt),
    ("LBracket", KeyCode::LBracket),
    ("LControl", KeyCode::LControl),
    ("LShift", KeyCode::LShift),
    ("LWin", KeyCode::LWin),
    ("MediaSelect", KeyCode::MediaSelect),
    ("MediaStop", KeyCode::MediaStop),
    ("Minus", KeyCode::Minus),
    ("Mute", KeyCode::Mute),
    ("NavigateForward", KeyCode::NavigateForward),
    ("NavigateBackward", KeyCode::NavigateBackward),
    ("NextTrack", KeyCode::NextTrack),
    ("Period", KeyCode::Period),
    ("PlayPause", KeyCode::PlayPause),
    ("Plus", KeyCode::Plus),
    ("PrevTrack", KeyCode::PrevTrack),
    ("RAlt", KeyCode::RAlt),
    ("RBracket", KeyCode::RBracket),
    ("RControl", KeyCode::RControl),
    ("RShift", KeyCode::RShift),
    ("RWin", KeyCode::RWin),
    ("Semicolon", KeyCode::Semicolon),
    ("Slash", KeyCode::Slash),
    ("Sleep", KeyCode::Sleep),
    ("Stop", KeyCode::Stop),
    ("Sysrq", KeyCode::Sysrq),
    ("Tab", KeyCode::Tab),
    ("Underline", KeyCode::Underline),
    ("VolumeDown", KeyCode::VolumeDown),
    ("VolumeUp", KeyCode::VolumeUp),
];

const GAMEPAD_BUTTON_MAPPING: [(&str, Button); 19] = [
    ("South", Button::South),
    ("East", Button::East),
    ("North", Button::North),
    ("West", Button::West),
    ("C", Button::C),
    ("Z", Button::Z),
    ("LeftTrigger", Button::LeftTrigger),
    ("LeftTrigger2", Button::LeftTrigger2),
    ("RightTrigger", Button::RightTrigger),
    ("RightTrigger2", Button::RightTrigger2),
    ("Select", Button::Select),
    ("Start", Button::Start),
    ("Mode", Button::Mode),
    ("LeftThumb", Button::LeftThumb),
    ("RightThumb", Button::RightThumb),
    ("DPadUp", Button::DPadUp),
    ("DPadDown", Button::DPadDown),
    ("DPadLeft", Button::DPadLeft),
    ("DPadRight", Button::DPadRight),
];

fn make_keyboard_button_map() -> HashMap<&'static str, KeyCode> {
    KEYBOARD_BUTTON_MAPPING.iter().cloned().collect()
}

fn make_valid_keyboard_button_vec() -> Vec<&'static str>{
    KEYBOARD_BUTTON_MAPPING.iter()
        .map(|x| x.0)
        .collect()
}

fn make_gamepad_button_map() -> HashMap<&'static str, Button> {
    GAMEPAD_BUTTON_MAPPING.iter().cloned().collect()
}

fn make_valid_gamepad_button_vec() -> Vec<&'static str>{
    GAMEPAD_BUTTON_MAPPING.iter()
        .map(|x| x.0)
        .collect()
}

// SampleId and BankId are used during config parsing, they are a human readable reference to
// specific banks/samples. They are translated to BankRef and SampleRef, which enable config structs
// to quickly look up other config structs by index.
type SampleId = String;
type BankId = String;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct BankRef {
    pub bank_index: usize,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct SampleRef {
    pub sample_index: usize,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct BankSampleRef {
    pub bank: BankRef,
    pub sample: SampleRef,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct SwitchRef {
    pub switch_index: usize,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SampleConfig {
    pub id: SampleId,
    pub file: PathBuf,

    // Cached //

    #[serde(skip)]
    pub bank_sample_ref: BankSampleRef,

    /// None if the config was embedded, Some if the config is from disk
    #[serde(skip)]
    pub file_resolved: Option<PathBuf>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BankConfig {
    pub id: BankId,
    /// If true, do not mute other samples when playing a sample in this bank
    #[serde(default)]
    pub poly: bool,
    #[serde(default)]
    pub samples: Vec<SampleConfig>,

    // Cached //

    #[serde(skip)]
    pub bank_ref: BankRef,
}

impl BankConfig {
    pub fn sample(&self, sample_ref: SampleRef) -> &SampleConfig {
        &self.samples[sample_ref.sample_index]
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Gamepad {
    pub device_id: Option<usize>,

    /// Must match an identifier present in GAMEPAD_BUTTON_MAPPING
    pub button: String,

    // Cached //

    #[serde(skip)]
    pub gilrs_button: Button,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct SwitchPlay {
    pub bank: BankId,
    pub sample: SampleId,

    // Cached //

    #[serde(skip)]
    pub bank_sample_ref: BankSampleRef,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct SwitchPlayRandom {
    pub bank: BankId,

    // Cached //

    #[serde(skip)]
    pub bank_ref: BankRef,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct SwitchPlayStep {
    pub bank: BankId,
    /// How many steps to skip to skip in the bank
    /// -1 = play the previous one
    /// 0 = repeat the last played sample
    /// 1 = play the next one
    /// 2 = play the next one, skipping a sample
    /// etc

    pub steps: i32,

    // Cached //

    #[serde(skip)]
    pub bank_ref: BankRef,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct SwitchConfig {
    // Switch trigger conditions //

    /// the title in the gui
    pub title: String,
    /// trigger based on a keyboard key
    pub key: Option<String>,
    /// trigger based on a gamepad button
    pub gamepad: Option<Gamepad>,

    // Actions //

    /// if true, stop all sounds
    #[serde(default)]
    pub stop_sounds: bool,

    /// Play a specific sample in a specific bank
    pub play: Option<SwitchPlay>,

    // Play a random sample in a bank
    pub play_random: Option<SwitchPlayRandom>,

    // Play a sample, relative in position to the sample previously played in a bank
    pub play_step: Option<SwitchPlayStep>,

    // Cached //

    #[serde(skip)]
    pub switch_ref: SwitchRef,

    /// same as `key` but translated to a KeyCode
    #[serde(skip)]
    pub key_code: Option<KeyCode>,
}

struct ConfigIdLookup {
    /// bank.id => BankRef
    bank_id_lookup: HashMap<String, BankRef>,

    /// sample.id => BankRef
    sample_id_lookup: HashMap<String, HashMap<String, BankSampleRef>>,
}

impl ConfigIdLookup {
    fn new(banks: &Vec<BankConfig>) -> Self {
        let bank_id_lookup = banks
            .iter()
            .map(|bank| (bank.id.clone(), bank.bank_ref))
            .collect();

        let sample_id_lookup = banks
            .iter()
            .map(|bank| {
                let samples = bank.samples
                    .iter()
                    .map(|sample| (sample.id.clone(), sample.bank_sample_ref))
                    .collect();

                (bank.id.clone(), samples)
            })
            .collect();

        ConfigIdLookup { bank_id_lookup, sample_id_lookup }
    }

    fn bank_id_to_ref(&self, bank_id: &str) -> Result<BankRef, ConfigError> {
        match self.bank_id_lookup.get(bank_id) {
            Some(bank_ref) => Ok(*bank_ref),
            None => Err(ConfigError::UnknownBankId {
                bank: bank_id.to_string(),
            }),
        }
    }

    fn sample_id_to_ref(&self, bank_id: &str, sample_id: &str) -> Result<BankSampleRef, ConfigError> {
        let result = self.sample_id_lookup
            .get(bank_id)
            .map(|map| map.get(sample_id));

        match result {
            None => Err(ConfigError::UnknownBankId {
                bank: bank_id.to_string(),
            }),
            Some(None) => Err(ConfigError::UnknownSampleId {
                bank: bank_id.to_string(),
                sample: sample_id.to_string(),
            }),
            Some(Some(bank_sample_ref)) => Ok(*bank_sample_ref),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub banks: Vec<BankConfig>,
    pub switches: Vec<SwitchConfig>,

    // Cached //

    /// The path that all other paths are relative to
    /// None if the config was embedded, Some if the config is from disk
    #[serde(skip)]
    pub resolve_path: Option<PathBuf>,

    #[serde(skip)]
    // keyboard key code => switch config reference
    keyboard_key_to_switch_lookup_table: HashMap<KeyCode, SwitchRef>,

    #[serde(skip)]
    // device id => gamepad button => switch config reference
    gamepad_button_to_switch_lookup_table: HashMap<Option<usize>, HashMap<Button, SwitchRef>>,

    #[serde(skip)]
    // for each switch that has a SwitchConfig.play (SwitchPlay) configuration, map the sample that it specifies to the switch
    sample_to_switch_play: HashMap<BankSampleRef, Vec<SwitchRef>>,

    #[serde(skip)]
    empty_switch_ref_vec: Vec<SwitchRef>, // serde will init this to Default::default() which will be an empty vec
}

impl Config {
    pub fn from_string(yaml_string: &str, resolve_path: Option<PathBuf>) -> Result<Config, ConfigError> {
        let mut config: Config = serde_yaml::from_str(yaml_string)?;
        config.resolve_path = resolve_path;

        config.resolve_refs()?;
        config.resolve_bank_paths();
        config.resolve_gamepad_button_mappings()?;
        config.resolve_keyboard_key_to_switch_lookup_table()?;
        config.resolve_gamepad_button_to_switch_lookup_table();
        config.resolve_sample_to_switch_play_lookup_table();

        Ok(config)
    }

    pub fn from_file(path: &Path) -> Result<Config, ConfigError> {
        let content = fs::read_to_string(&path)?;

        // all paths defined in the config file, are relative to the directory the config file is in
        let mut resolve_path = path.to_path_buf();
        resolve_path.pop();
        Ok(Config::from_string(&content, Some(resolve_path))?)
    }

    fn resolve_refs(&mut self) -> Result<(), ConfigError> {
        for (bank_index, bank_config) in &mut self.banks.iter_mut().enumerate() {
            bank_config.bank_ref.bank_index = bank_index;

            for (sample_index, sample_config) in &mut bank_config.samples.iter_mut().enumerate() {
                sample_config.bank_sample_ref.bank.bank_index = bank_index;
                sample_config.bank_sample_ref.sample.sample_index = sample_index;
            }
        }

        for (switch_index, switch_config) in &mut self.switches.iter_mut().enumerate() {
            switch_config.switch_ref.switch_index = switch_index;
        }

        let lookup = ConfigIdLookup::new(&self.banks);

        for switch_config in &mut self.switches.iter_mut() {
            if let Some(play) = &mut switch_config.play.as_mut() {
                play.bank_sample_ref = lookup.sample_id_to_ref(&play.bank, &play.sample)?;
            }

            if let Some(play) = &mut switch_config.play_random.as_mut() {
                play.bank_ref = lookup.bank_id_to_ref(&play.bank)?;
            }

            if let Some(play) = &mut switch_config.play_step.as_mut() {
                play.bank_ref = lookup.bank_id_to_ref(&play.bank)?;
            }
        }

        Ok(())
    }

    fn resolve_bank_paths(&mut self) {
        let resolve_path = &self.resolve_path;
        if resolve_path.is_none() {
            return;
        }
        let resolve_path = resolve_path.as_ref().unwrap();

        for bank_config in &mut self.banks {
            for sample in &mut bank_config.samples {
                let mut resolved_file = PathBuf::from(resolve_path);
                resolved_file.push(&sample.file);
                sample.file_resolved = Some(resolved_file);
            }
        }
    }

    fn resolve_gamepad_button_mappings(&mut self) -> Result<(), ConfigError> {
        let mapping = make_gamepad_button_map();
        for switch_config in &mut self.switches {
            if let Some(gamepad) = &mut switch_config.gamepad {
                let gilrs_button = match mapping.get(&gamepad.button.as_str()) {
                    None => {
                        return Err(ConfigError::UnknownGamepadButton {
                            button: String::from(&gamepad.button),
                            allowed_values: make_valid_gamepad_button_vec().join(", "),
                        });
                    },
                    Some(v) => v,
                };
                gamepad.gilrs_button = *gilrs_button;
            }
        }

        Ok(())
    }

    fn resolve_keyboard_key_to_switch_lookup_table(&mut self) -> Result<(), ConfigError> {
        let mapping = make_keyboard_button_map();
        let mut lookup_table = HashMap::new();

        for switch_config in (&mut self.switches).into_iter() {
            if let Some(key) = &switch_config.key {
                match mapping.get(key.as_str()) {
                    Some(key_code) => {
                        switch_config.key_code = Some(*key_code);
                    },
                    None => {
                        return Err(ConfigError::UnknownKeyboardButton {
                            button: key.to_string(),
                            allowed_values: make_valid_keyboard_button_vec().join(", "),
                        });
                    },
                }
            }
        }

        for switch_config in (&self.switches).into_iter() {
            if let Some(key_code) = &switch_config.key_code {
                lookup_table.insert(*key_code, switch_config.switch_ref);
            }
        }

        self.keyboard_key_to_switch_lookup_table = lookup_table;
        Ok(())
    }

    fn resolve_gamepad_button_to_switch_lookup_table(&mut self) {
        let gamepad_configs = (&self.switches)
            .into_iter()
            .filter_map(|switch_config| {
                match &switch_config.gamepad {
                    Some(gamepad) => Some((gamepad, switch_config.switch_ref)),
                    None => None,
                }
            });

        let mut lookup_table = HashMap::new();

        for (gamepad_config, switch_ref) in gamepad_configs {
            if !lookup_table.contains_key(&gamepad_config.device_id) {
                lookup_table.insert(gamepad_config.device_id, HashMap::new());
            }

            let map = lookup_table.get_mut(&gamepad_config.device_id).unwrap();
            map.insert(gamepad_config.gilrs_button, switch_ref);
        }

        self.gamepad_button_to_switch_lookup_table = lookup_table;
    }

    fn resolve_sample_to_switch_play_lookup_table(&mut self) {
        let mut lookup_table = HashMap::new();

        for switch in &self.switches {
            if let Some(play) = &switch.play {
                let list = lookup_table.entry(play.bank_sample_ref).or_insert_with(|| Vec::new());
                list.push(switch.switch_ref);
            }
        }

        self.sample_to_switch_play = lookup_table;
    }

    pub fn find_switch_for_keyboard_key(&self, key: KeyCode) -> Option<&SwitchConfig> {
        match self.keyboard_key_to_switch_lookup_table.get(&key) {
            Some(switch_ref) => Some(&self.switches[switch_ref.switch_index]),
            None => None,
        }
    }

    pub fn find_switch_for_gamepad_button(&self, device_id: usize, button: Button) -> Option<&SwitchConfig> {
        // first try to find a switch configured for a specific gamepad device
        let switch_ref = match self.gamepad_button_to_switch_lookup_table.get(&Some(device_id)) {
            Some(map) => map.get(&button),
            None => None,
        };

        // Then try to find a switch configured for all gamepad devices
        let switch_ref = match switch_ref {
            Some(v) => Some(v),
            None => {
                match self.gamepad_button_to_switch_lookup_table.get(&None) {
                    Some(map) => map.get(&button),
                    None => None,
                }
            }
        };

        match switch_ref {
            Some(switch_ref) => Some(&self.switches[switch_ref.switch_index]),
            None => None,
        }
    }

    pub fn find_switch_play_for_sample(&self, bank_sample_ref: BankSampleRef) -> &Vec<SwitchRef> {
        if let Some(list) = self.sample_to_switch_play.get(&bank_sample_ref) {
            list
        }
        else {
            &self.empty_switch_ref_vec
        }
    }

    pub fn switch(&self, switch_ref: SwitchRef) -> &SwitchConfig {
        // this will crash if switch_ref.switch_index is out of bounds, however the expectation is
        // that a SwitchRef instance is always valid.
        &self.switches[switch_ref.switch_index]
    }

    pub fn bank(&self, bank_ref: BankRef) -> &BankConfig {
        &self.banks[bank_ref.bank_index]
    }

    pub fn sample(&self, bank_sample_ref: BankSampleRef) -> (&BankConfig, &SampleConfig) {
        let bank = self.bank(bank_sample_ref.bank);
        let sample = bank.sample(bank_sample_ref.sample);
        (bank, sample)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{Config, BankConfig, BankRef, SampleConfig, SampleRef, BankSampleRef, SwitchConfig, SwitchRef, SwitchPlay, SwitchPlayRandom, SwitchPlayStep, Gamepad};
    use std::path::{PathBuf};
    use gilrs::Button;
    use iced::keyboard::KeyCode;
    use pretty_assertions::{assert_eq};
    use crate::error::ConfigError;

    fn test_path(extra_parts: &[&str]) -> PathBuf {
        let path_sep = std::path::MAIN_SEPARATOR.to_string();
        [path_sep.as_str(), "test", "path"].iter().chain(extra_parts.iter()).collect()
    }

    #[test]
    fn successful_config_parsing() {
        let config_source = r###"
banks:
  - id: bankA
    # poly option is false by default
    poly: true
    samples:
      # Multiple samples
      - id: foo1
        file: foo1.mp3
      - id: foo2
        file: foo2.wav
      - id: foo3
        file: foo3.ogg
      - id: foo4
        file: foo4.flac

  - id: bankB
    # poly option is not set here
    samples:
      # sample with id "foo1" also exists in "bankA". They should not interfere
      - id: foo1
        file: foo1-bankB.mp3

switches:
  - title: play option
    play:
      bank: bankB
      sample: foo1

  - title: playRandom option
    playRandom:
      bank: bankB

  - title: playStep option
    playStep:
      bank: bankA
      steps: 1

  - title: stopSounds option
    stopSounds: true

  - title: Only the required fields

  - title: keyboard key
    key: X

  - title: gamepad button on any device
    gamepad:
      button: North

  - title: gamepad button, specific device
    gamepad:
      deviceId: 123
      button: North

  - title: another play option, same actions as the first
    play:
      bank: bankB
      sample: foo1
"###;
        let config = Config::from_string(config_source, Some(test_path(&[]))).unwrap();
        assert_eq!(config, Config {
            banks: vec![
                BankConfig {
                    id: "bankA".to_string(),
                    poly: true,
                    samples: vec![
                        SampleConfig {
                            id: "foo1".to_string(),
                            file: PathBuf::from("foo1.mp3"),
                            bank_sample_ref: BankSampleRef {
                                bank: BankRef { bank_index: 0 },
                                sample: SampleRef { sample_index: 0 },
                            },
                            file_resolved: Some(test_path(&["foo1.mp3"])),
                        },
                        SampleConfig {
                            id: "foo2".to_string(),
                            file: PathBuf::from("foo2.wav"),
                            bank_sample_ref: BankSampleRef {
                                bank: BankRef { bank_index: 0 },
                                sample: SampleRef { sample_index: 1 },
                            },
                            file_resolved: Some(test_path(&["foo2.wav"])),
                        },
                        SampleConfig {
                            id: "foo3".to_string(),
                            file: PathBuf::from("foo3.ogg"),
                            bank_sample_ref: BankSampleRef {
                                bank: BankRef { bank_index: 0 },
                                sample: SampleRef { sample_index: 2 },
                            },
                            file_resolved: Some(test_path(&["foo3.ogg"])),
                        },
                        SampleConfig {
                            id: "foo4".to_string(),
                            file: PathBuf::from("foo4.flac"),
                            bank_sample_ref: BankSampleRef {
                                bank: BankRef { bank_index: 0 },
                                sample: SampleRef { sample_index: 3 },
                            },
                            file_resolved: Some(test_path(&["foo4.flac"])),
                        },
                    ],
                    bank_ref: BankRef { bank_index: 0 },
                },
                BankConfig {
                    id: "bankB".to_string(),
                    poly: false,
                    samples: vec![
                        SampleConfig {
                            id: "foo1".to_string(),
                            file: PathBuf::from("foo1-bankB.mp3"),
                            bank_sample_ref: BankSampleRef {
                                bank: BankRef { bank_index: 1 },
                                sample: SampleRef { sample_index: 0 },
                            },
                            file_resolved: Some(test_path(&["foo1-bankB.mp3"])),
                        },
                    ],
                    bank_ref: BankRef { bank_index: 1 },
                },
            ],
            switches: vec![
                SwitchConfig {
                    title: "play option".to_string(),
                    key: None,
                    gamepad: None,
                    stop_sounds: false,
                    play: Some(SwitchPlay {
                        bank: "bankB".to_string(),
                        sample: "foo1".to_string(),
                        bank_sample_ref: BankSampleRef {
                            bank: BankRef { bank_index: 1 },
                            sample: SampleRef { sample_index: 0 },
                        }
                    }),
                    play_random: None,
                    play_step: None,
                    switch_ref: SwitchRef { switch_index: 0 },
                    key_code: None
                },
                SwitchConfig {
                    title: "playRandom option".to_string(),
                    key: None,
                    gamepad: None,
                    stop_sounds: false,
                    play: None,
                    play_random: Some(
                        SwitchPlayRandom {
                            bank: "bankB".to_string(),
                            bank_ref: BankRef { bank_index: 1 },
                        },
                    ),
                    play_step: None,
                    switch_ref: SwitchRef { switch_index: 1 },
                    key_code: None,
                },
                SwitchConfig {
                    title: "playStep option".to_string(),
                    key: None,
                    gamepad: None,
                    stop_sounds: false,
                    play: None,
                    play_random: None,
                    play_step: Some(
                        SwitchPlayStep {
                            bank: "bankA".to_string(),
                            steps: 1,
                            bank_ref: BankRef { bank_index: 0 },
                        },
                    ),
                    switch_ref: SwitchRef { switch_index: 2 },
                    key_code: None,
                },
                SwitchConfig {
                    title: "stopSounds option".to_string(),
                    key: None,
                    gamepad: None,
                    stop_sounds: true,
                    play: None,
                    play_random: None,
                    play_step: None,
                    switch_ref: SwitchRef { switch_index: 3 },
                    key_code: None,
                },
                SwitchConfig {
                    title: "Only the required fields".to_string(),
                    key: None,
                    gamepad: None,
                    stop_sounds: false,
                    play: None,
                    play_random: None,
                    play_step: None,
                    switch_ref: SwitchRef { switch_index: 4 },
                    key_code: None,
                },
                SwitchConfig {
                    title: "keyboard key".to_string(),
                    key: Some("X".to_string()),
                    gamepad: None,
                    stop_sounds: false,
                    play: None,
                    play_random: None,
                    play_step: None,
                    switch_ref: SwitchRef { switch_index: 5 },
                    key_code: Some(KeyCode::X),
                },
                SwitchConfig {
                    title: "gamepad button on any device".to_string(),
                    key: None,
                    gamepad: Some(
                        Gamepad {
                            device_id: None,
                            button: "North".to_string(),
                            gilrs_button: Button::North,
                        },
                    ),
                    stop_sounds: false,
                    play: None,
                    play_random: None,
                    play_step: None,
                    switch_ref: SwitchRef { switch_index: 6 },
                    key_code: None,
                },
                SwitchConfig {
                    title: "gamepad button, specific device".to_string(),
                    key: None,
                    gamepad: Some(
                        Gamepad {
                            device_id: Some(123),
                            button: "North".to_string(),
                            gilrs_button: Button::North,
                        },
                    ),
                    stop_sounds: false,
                    play: None,
                    play_random: None,
                    play_step: None,
                    switch_ref: SwitchRef { switch_index: 7 },
                    key_code: None,
                },
                SwitchConfig {
                    title: "another play option, same actions as the first".to_string(),
                    key: None,
                    gamepad: None,
                    stop_sounds: false,
                    play: Some(SwitchPlay {
                        bank: "bankB".to_string(),
                        sample: "foo1".to_string(),
                        bank_sample_ref: BankSampleRef {
                            bank: BankRef { bank_index: 1 },
                            sample: SampleRef { sample_index: 0 },
                        }
                    }),
                    play_random: None,
                    play_step: None,
                    switch_ref: SwitchRef { switch_index: 8 },
                    key_code: None
                },
            ],
            resolve_path: Some(test_path(&[])),

            keyboard_key_to_switch_lookup_table: vec![
                (KeyCode::X, SwitchRef { switch_index: 5 }),
            ].into_iter().collect(),

            gamepad_button_to_switch_lookup_table: vec![
                (
                    None,
                    vec![
                        (Button::North, SwitchRef { switch_index: 6 }),
                    ].into_iter().collect(),
                ),
                (
                    Some(123),
                    vec![
                        (Button::North, SwitchRef { switch_index: 7 }),
                    ].into_iter().collect(),
                ),
            ].into_iter().collect(),

            sample_to_switch_play: vec![
                (
                    BankSampleRef {
                        bank: BankRef { bank_index: 1 },
                        sample: SampleRef { sample_index: 0 },
                    },
                    vec![
                        SwitchRef { switch_index: 0 },
                        SwitchRef { switch_index: 8 },
                    ],
                )
            ].into_iter().collect(),
            empty_switch_ref_vec: vec![],
        });
    }

    #[test]
    fn find_switch_for_keyboard_key() {
        let config_source = r###"
banks: []
switches:
  - title: Key a
    key: A

  # should override the first switch
  - title: Key a (again)
    key: A

  - title: Key b
    key: B

  - title: No key
"###;

        let config = Config::from_string(config_source, Some(test_path(&[]))).unwrap();
        assert!(config.find_switch_for_keyboard_key(KeyCode::Z).is_none());
        assert_eq!(config.find_switch_for_keyboard_key(KeyCode::A).unwrap().title, "Key a (again)");
        assert_eq!(config.find_switch_for_keyboard_key(KeyCode::B).unwrap().title, "Key b");
    }

    #[test]
    fn find_switch_for_gamepad_button() {
        let config_source = r###"
banks: []
switches:
  - title: South, no device filter
    gamepad:
      button: South

  # should override the first switch
  - title: South, no device filter (again)
    gamepad:
      button: South

  - title: North, device 123
    gamepad:
      deviceId: 123
      button: North

  # Should have less priority because it specifies no device
  - title: North, no device filter
    gamepad:
      button: North

  # The only entry that specifies West
  - title: West, device 123
    gamepad:
      deviceId: 123
      button: West
"###;
        let config = Config::from_string(config_source, Some(test_path(&[]))).unwrap();

        assert!(config.find_switch_for_gamepad_button(10, Button::West).is_none());
        assert_eq!(config.find_switch_for_gamepad_button(123, Button::West).unwrap().title, "West, device 123");

        assert_eq!(config.find_switch_for_gamepad_button(123, Button::South).unwrap().title, "South, no device filter (again)");

        assert_eq!(config.find_switch_for_gamepad_button(123, Button::North).unwrap().title, "North, device 123");
        assert_eq!(config.find_switch_for_gamepad_button(456, Button::North).unwrap().title, "North, no device filter");

        // Start button is not specified at all
        assert!(config.find_switch_for_gamepad_button(10, Button::Start).is_none());
    }

    #[test]
    fn config_with_yaml_syntax_error() {
        let config_source = "bla'[ ";
        Config::from_string(config_source, Some(test_path(&[]))).unwrap_err();
    }

    #[test]
    fn config_with_mismatched_type() {
        let config_source = r###"
switches: this should be an array
"###;
        let error = Config::from_string(config_source, Some(test_path(&[]))).unwrap_err();
        let error_message = format!("{}", error);
        assert!(error_message.contains("invalid type: string"));
    }

    #[test]
    fn config_with_invalid_bank_id() {
        let config_source = r###"
banks:
  - id: mybank
switches:
  - title: invalid bank id
    playRandom:
      bank: invalid
"###;
        let error = Config::from_string(config_source, Some(test_path(&[]))).unwrap_err();
        match error {
            ConfigError::UnknownBankId { bank } => {
                assert_eq!(bank.as_str(), "invalid");
            }
            _ => {
                panic!("Expected error to be ConfigError::UnknownBankId");
            }
        }
    }

    #[test]
    fn config_with_invalid_sample_id() {
        let config_source = r###"
banks:
  - id: mybank
    samples:
      - id: mysample
        file: foo.mp3

switches:
  - title: invalid bank id
    play:
      bank: mybank
      sample: invalid
"###;
        let error = Config::from_string(config_source, Some(test_path(&[]))).unwrap_err();
        match error {
            ConfigError::UnknownSampleId { bank, sample } => {
                assert_eq!(bank.as_str(), "mybank");
                assert_eq!(sample.as_str(), "invalid");
            }
            _ => {
                panic!("Expected error to be ConfigError::UnknownSampleId");
            }
        }
    }
}
