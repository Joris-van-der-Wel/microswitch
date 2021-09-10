use crate::config::{Config, SwitchRef};
use iced::{button, Button, Column, Text, Settings, Error, Element, Align, Length, Container, Application, executor, Clipboard, Command, Subscription, Background, Color, Vector};
use iced_native::{Event, keyboard, window};
use iced::time::{every as iced_time_every};
use iced::window::Icon;
use crate::sound_thread::{SoundThread, SoundThreadRpc, SoundThreadEvent};
use crate::gamepad_thread::{GamepadThread};
use std::time::{Duration, Instant};
use std::sync::mpsc::Receiver;

pub struct ApplicationFlags {
    config: Config,
    sound_thread: SoundThread,
    gamepad_thread: GamepadThread,
    sound_thread_event_receiver: Receiver<SoundThreadEvent>,
}

// Whenever a sample is played, the corresponding button will be highlighted temporarily.
// These constant specify the duration and colour of this highlight.
// Buttons that only play a random or a stepped sample, will not be highlighted.
const BUTTON_PLAY_FEEDBACK_DURATION: u128 = 1000;
const BUTTON_PLAY_FEEDBACK_FROM: (f32, f32, f32) = (51.0 / 255.0, 147.0 / 255.0, 129.9 / 255.0);
const BUTTON_PLAY_FEEDBACK_TO: (f32, f32, f32) = (0.87, 0.87, 0.87);

fn interpolate(from: f32, to: f32, progress: f32) -> f32 {
    (to - from) * progress + from
}

struct ButtonStyleSheet {
    last_played_ago: Option<Duration>,
}

impl button::StyleSheet for ButtonStyleSheet {
    fn active(&self) -> button::Style {
        let play_feedback_progress: f32 = match self.last_played_ago {
            None => 1.0,
            Some(last_played_ago) => {
                let last_played_ago = last_played_ago.as_millis();
                if last_played_ago >= BUTTON_PLAY_FEEDBACK_DURATION {
                    1.0
                }
                else {
                    last_played_ago as f32 / BUTTON_PLAY_FEEDBACK_DURATION as f32
                }
            }
        };

        let background_color= Color::from_rgb(
            interpolate(BUTTON_PLAY_FEEDBACK_FROM.0, BUTTON_PLAY_FEEDBACK_TO.0, play_feedback_progress),
            interpolate(BUTTON_PLAY_FEEDBACK_FROM.1, BUTTON_PLAY_FEEDBACK_TO.1, play_feedback_progress),
            interpolate(BUTTON_PLAY_FEEDBACK_FROM.2, BUTTON_PLAY_FEEDBACK_TO.2, play_feedback_progress),
        );

        button::Style {
            shadow_offset: Vector::new(0.0, 0.0),
            background: Some(Background::Color(background_color)),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: [0.7, 0.7, 0.7].into(),
            text_color: Color::BLACK,
        }
    }
}

struct PlayButtonState {
    switch_title: String,
    button: button::State,
    /// Is the keyboard key currently down? A sample should only play for a single press, without repeating.
    key_held_down: bool,
    /// The last time the corresponding sample (the one specified by config.switches[button_index].play.unwrap().bank_sample_ref) has been played.
    last_played_at: Option<Instant>,
}

pub struct MyApplication {
    config: Config,
    sound_thread: Option<SoundThread>,
    sound_thread_rpc: SoundThreadRpc,
    gamepad_thread: Option<GamepadThread>,
    sound_thread_event_receiver: Receiver<SoundThreadEvent>,
    /// The state of each rendered play button. Each configured switch (SwitchConfig) has a 1:1
    /// correspondence with a button, at the same index.
    play_buttons: Vec<PlayButtonState>,
    now: Instant,
    should_exit: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
    EventOccurred(Event),
    PlayButtonPressed(usize), // (index)
}

impl MyApplication {
    fn exit(&mut self) {
        if self.should_exit {
            return;
        }

        let sound_thread = self.sound_thread.take().unwrap();
        let gamepad_thread = self.gamepad_thread.take().unwrap();

        if let Err(err) = sound_thread.stop() {
            eprintln!("Error while stopping SoundThread: {:?}", err);
        }
        if let Err(err) = gamepad_thread.stop() {
            eprintln!("Error while stopping GamepadThread: {:?}", err);
        }

        self.should_exit = true;
    }

    fn switch_pressed(&self, switch_ref: SwitchRef) {
        if let Err(err) = self.sound_thread_rpc.switch_pressed(switch_ref) {
            eprintln!("Error sending switch_pressed to SoundThread {}", err);
        }
    }
}

impl Application for MyApplication {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ApplicationFlags;

    fn new(flags: ApplicationFlags) -> (MyApplication, Command<Self::Message>) {
        let config = flags.config;
        let sound_thread = flags.sound_thread;
        let sound_thread_rpc = SoundThreadRpc::new(&sound_thread);
        let gamepad_thread = flags.gamepad_thread;
        let sound_thread_event_receiver = flags.sound_thread_event_receiver;
        let switches = &config.switches;

        let play_buttons = switches
            .into_iter()
            .map(|switch_config| PlayButtonState {
                switch_title: switch_config.title.clone(),
                button: button::State::new(),
                key_held_down: false,
                last_played_at: None,
            })
            .collect();

        let app = MyApplication {
            config,
            sound_thread: Some(sound_thread),
            sound_thread_rpc,
            gamepad_thread: Some(gamepad_thread),
            sound_thread_event_receiver,
            now: Instant::now(),
            play_buttons,
            should_exit: false,
        };

        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from(concat!("μSwitch ", env!("CARGO_PKG_VERSION")))
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Self::Message>{
        match message {
            Message::Tick(now) => {
                self.now = now;

                for event in self.sound_thread_event_receiver.try_iter() {
                    match event {
                        SoundThreadEvent::PlayedSample(bank_sample_ref) => {
                            let switches = self.config.find_switch_play_for_sample(bank_sample_ref);

                            for switch_ref in switches {
                                let button_state = &mut self.play_buttons[switch_ref.switch_index];
                                button_state.last_played_at = Some(self.now);
                            }
                        },
                    }
                }
            },
            Message::PlayButtonPressed(index) => {
                // for now a button is created for each SwitchConfig, so we can just use the button
                // index to look up the SwitchConfig
                let switch_ref = self.config.switches[index].switch_ref;
                self.switch_pressed(switch_ref);
            },
            Message::EventOccurred(Event::Window(window::Event::CloseRequested)) => {
                self.exit();
            },
            Message::EventOccurred(Event::Keyboard(keyboard::Event::KeyPressed { key_code, modifiers: _ })) => {
                if let Some(switch_config) = self.config.find_switch_for_keyboard_key(key_code) {
                    let switch_ref = switch_config.switch_ref;
                    let button_state = &mut self.play_buttons[switch_ref.switch_index];
                    let was_held_down = button_state.key_held_down;
                    button_state.key_held_down = true;

                    if !was_held_down {
                        self.switch_pressed(switch_ref);
                    }
                }
            },
            Message::EventOccurred(Event::Keyboard(keyboard::Event::KeyReleased { key_code, modifiers: _ })) => {
                println!("Keyboard release {:?}", key_code);

                if let Some(switch_config) = self.config.find_switch_for_keyboard_key(key_code) {
                    let button_state = &mut self.play_buttons[switch_config.switch_ref.switch_index];
                    button_state.key_held_down = false;
                }
            },
            _ => {},
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced_native::subscription::events().map(Message::EventOccurred),
            iced_time_every(Duration::from_millis(8)).map(|_| Message::Tick(Instant::now())),
        ])
    }

    fn view(&mut self) -> Element<Message> {
        let now = self.now;
        let play_buttons = self.play_buttons.iter_mut();

        let mut column = Column::new()
            .padding(20)
            .align_items(Align::Center);

        let mut index = 0;
        for play_button in play_buttons {
            let stylesheet = ButtonStyleSheet {
                last_played_ago: play_button.last_played_at.map(|ago| now.duration_since(ago)),
            };

            let button = Button::new(&mut play_button.button, Text::new(&play_button.switch_title))
                .width(Length::Fill)
                .style(stylesheet)
                .on_press(Message::PlayButtonPressed(index));

            column = column.push(
                Container::new(button)
                .width(Length::Fill).padding(5)
            );
            index += 1;
        }

        column.into()
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }
}

fn make_icon() -> Icon {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/microswitch-icon-32-rgba"));
    let bytes = bytes.to_vec();
    Icon::from_rgba(bytes, 32, 32).expect("Failed to load window icon")
}

pub fn run_application(
    config: &Config,
    sound_thread: SoundThread,
    gamepad_thread: GamepadThread,
    sound_thread_event_receiver: Receiver<SoundThreadEvent>,
) -> Result<(), Error> {
    let config = config.clone();
    let flags = ApplicationFlags { config, sound_thread, gamepad_thread, sound_thread_event_receiver };
    let mut settings = Settings::with_flags(flags);

    // this we will handle ourselves so that we can do cleanup
    settings.exit_on_close_request = false;

    settings.window.icon = Some(make_icon());

    // this function will call process::exit() unless there was a startup error
    MyApplication::run(settings)
}
