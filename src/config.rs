use crate::error::ConfigError;
use gilrs::Button;
use iced::keyboard::KeyCode;
use serde::{Deserialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::{fs, array};


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
    array::IntoIter::new(KEYBOARD_BUTTON_MAPPING)
        .map(|x| x.0)
        .collect()
}

fn make_gamepad_button_map() -> HashMap<&'static str, Button> {
    GAMEPAD_BUTTON_MAPPING.iter().cloned().collect()
}

fn make_valid_gamepad_button_vec() -> Vec<&'static str>{
    array::IntoIter::new(GAMEPAD_BUTTON_MAPPING)
        .map(|x| x.0)
        .collect()
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SampleConfig {
    pub file: PathBuf,
    #[serde(skip)]
    pub file_resolved: PathBuf,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Gamepad {
    pub device_id: Option<usize>,
    pub button: String,
    #[serde(skip)]
    pub gilrs_button: Button,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct SwitchConfig {
    #[serde(skip)]
    pub index: usize,
    pub title: String,
    pub key: Option<String>,
    #[serde(skip)]
    pub key_code: Option<KeyCode>,
    pub gamepad: Option<Gamepad>,
    #[serde(default = "Vec::new")]
    pub samples: Vec<SampleConfig>,
    #[serde(default="bool::default")]
    pub stop_sounds: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub switches: Vec<SwitchConfig>,

    // The path that all other paths are relative to
    #[serde(skip)]
    pub resolve_path: PathBuf,

    #[serde(skip)]
    keyboard_key_to_switch_lookup_table: HashMap<KeyCode, usize>,

    #[serde(skip)]
    gamepad_button_to_switch_lookup_table: HashMap<Option<usize>, HashMap<Button, usize>>,
}

impl Config {
    fn from_string(yaml_string: &str, resolve_path: PathBuf) -> Result<Config, ConfigError> {
        let mut config: Config = serde_yaml::from_str(yaml_string)?;
        config.resolve_path = resolve_path;

        config.resolve_switch_index();
        config.resolve_switch_paths();
        config.resolve_gamepad_button_mappings()?;
        config.resolve_keyboard_key_to_switch_lookup_table()?;
        config.resolve_gamepad_button_to_switch_lookup_table();

        Ok(config)
    }

    pub fn from_file(path: &Path) -> Result<Config, ConfigError> {
        let content = fs::read_to_string(&path)?;

        // all paths defined in the config file, are relative to the directory the config file is in
        let mut resolve_path = path.to_path_buf();
        resolve_path.pop();
        Ok(Config::from_string(&content, resolve_path)?)
    }

    fn resolve_switch_index(&mut self) {
        for (index, switch_config) in &mut self.switches.iter_mut().enumerate() {
            switch_config.index = index;
        }
    }

    fn resolve_switch_paths(&mut self) {
        for switch_config in &mut self.switches {
            for sample in &mut switch_config.samples {
                let mut resolved_file = PathBuf::from(&self.resolve_path);
                resolved_file.push(&sample.file);
                sample.file_resolved = resolved_file;
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

        for (index, switch_config) in (&self.switches).into_iter().enumerate() {
            if let Some(key_code) = &switch_config.key_code {
                lookup_table.insert(*key_code, index);
            }
        }

        self.keyboard_key_to_switch_lookup_table = lookup_table;
        Ok(())
    }

    fn resolve_gamepad_button_to_switch_lookup_table(&mut self) {
        let gamepad_configs = (&self.switches)
            .into_iter()
            .enumerate()
            .filter_map(|(index, switch_config)| {
                match &switch_config.gamepad {
                    Some(gamepad) => Some((gamepad, index)),
                    None => None,
                }
            });

        let mut lookup_table = HashMap::new();

        for (gamepad_config, switch_index) in gamepad_configs {
            if !lookup_table.contains_key(&gamepad_config.device_id) {
                lookup_table.insert(gamepad_config.device_id, HashMap::new());
            }

            let map = lookup_table.get_mut(&gamepad_config.device_id).unwrap();
            map.insert(gamepad_config.gilrs_button, switch_index);
        }

        self.gamepad_button_to_switch_lookup_table = lookup_table;
    }

    pub fn find_switch_for_keyboard_key(&self, key: KeyCode) -> Option<&SwitchConfig> {
        match self.keyboard_key_to_switch_lookup_table.get(&key) {
            Some(index) => Some(&self.switches[*index]),
            None => None,
        }
    }

    pub fn find_switch_for_gamepad_button(&self, device_id: usize, button: Button) -> Option<&SwitchConfig> {
        // first try to find a switch configured for a specific gamepad device
        let index = match self.gamepad_button_to_switch_lookup_table.get(&Some(device_id)) {
            Some(map) => map.get(&button),
            None => None,
        };

        // Then try to find a switch configured for all gamepad devices
        let index = match index {
            Some(v) => Some(v),
            None => {
                match self.gamepad_button_to_switch_lookup_table.get(&None) {
                    Some(map) => map.get(&button),
                    None => None,
                }
            }
        };

        match index {
            Some(index) => Some(&self.switches[*index]),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{Config};
    use std::path::{Path, PathBuf};
    use std::collections::HashMap;
    use gilrs::Button;
    use iced::keyboard::KeyCode;

    fn test_path() -> PathBuf { PathBuf::from("/test/path") }

    #[test]
    fn successful_config_parsing() {
        let config_source = r###"
switches:
  - title: Foo
    key: Space
    gamepad:
      button: South
    samples:
      - file: foo1.mp3
      - file: foo2.wav
      - file: foo3.ogg
      - file: foo4.flac

  - title: Bar
    key: 1
    gamepad:
      deviceId: 123
      button: North
    stopSounds: true
    samples:
      - file: bar.mp3

  - title: without a key / gamepad
    samples:
      - file: bar.mp3

  - title: another one
    gamepad:
      button: West
    samples:
      - file: bar.mp3

  - title: Only the required fields
"###;
        let config = Config::from_string(config_source, test_path()).unwrap();
        assert_eq!(config.switches.len(), 5);
        assert_eq!(config.switches[0].title, "Foo");
        assert_eq!(config.switches[0].key, Some(String::from("Space")));
        assert_eq!(config.switches[0].key_code, Some(KeyCode::Space));
        assert_eq!(config.switches[0].samples.len(), 4);
        assert_eq!(config.switches[0].samples[0].file, PathBuf::from("foo1.mp3"));
        assert_eq!(config.switches[0].samples[0].file_resolved, PathBuf::from("/test/path/foo1.mp3"));
        assert_eq!(config.switches[0].samples[0].file, PathBuf::from("foo1.mp3"));
        assert_eq!(config.switches[0].samples[3].file_resolved, PathBuf::from("/test/path/foo4.flac"));
        assert_eq!(config.switches[0].stop_sounds, false);

        assert_eq!(config.switches[1].title, "Bar");
        assert_eq!(config.switches[1].key, Some(String::from("1")));
        assert_eq!(config.switches[1].key_code, Some(KeyCode::Key1));
        assert_eq!(config.switches[1].samples.len(), 1);
        assert_eq!(config.switches[1].stop_sounds, true);

        assert_eq!(config.switches[2].key, None);

        {
            let mut expected: HashMap<KeyCode, usize> = HashMap::new();
            expected.insert(KeyCode::Space, 0);
            expected.insert(KeyCode::Key1, 1);
            assert_eq!(config.keyboard_key_to_switch_lookup_table, expected);
        }

        {
            let mut expected: HashMap<Option<usize>, HashMap<Button, usize>> = HashMap::new();
            let mut device_none = HashMap::new();
            device_none.insert(Button::South, 0);
            device_none.insert(Button::West, 3);
            expected.insert(None, device_none);

            let mut device_123 = HashMap::new();
            device_123.insert(Button::North, 1);
            expected.insert(Some(123), device_123);

            assert_eq!(config.gamepad_button_to_switch_lookup_table, expected);
        }
    }

    #[test]
    fn find_switch_for_keyboard_key() {
        let config_source = r###"
switches:
  - title: Key a
    key: A
    samples: []

  # should override the first switch
  - title: Key a (again)
    key: A
    samples: []

  - title: Key b
    key: B
    samples: []

  - title: No key
    samples: []
"###;

        let config = Config::from_string(config_source, test_path()).unwrap();
        assert!(config.find_switch_for_keyboard_key(KeyCode::Z).is_none());
        assert_eq!(config.find_switch_for_keyboard_key(KeyCode::A).unwrap().title, "Key a (again)");
        assert_eq!(config.find_switch_for_keyboard_key(KeyCode::B).unwrap().title, "Key b");
    }

    #[test]
    fn find_switch_for_gamepad_button() {
        let config_source = r###"
switches:
  - title: South, no device filter
    gamepad:
      button: South
    samples: []

  # should override the first switch
  - title: South, no device filter (again)
    gamepad:
      button: South
    samples: []

  - title: North, device 123
    gamepad:
      deviceId: 123
      button: North
    samples: []

  # Should have less priority because it specifies no device
  - title: North, no device filter
    gamepad:
      button: North
    samples: []

  # The only entry that specifies West
  - title: West, device 123
    gamepad:
      deviceId: 123
      button: West
    samples: []
"###;
        let config = Config::from_string(config_source, test_path()).unwrap();

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
        Config::from_string(config_source, test_path()).unwrap_err();
    }

    #[test]
    fn config_with_mismatched_type() {
        let config_source = r###"
switches:
  - title: Foo
    samples: this should be an array
"###;
        let error = Config::from_string(config_source, test_path()).unwrap_err();
        let error_message = format!("{}", error);
        assert!(error_message.contains("invalid type: string"));
    }
}
