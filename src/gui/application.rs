use iced::{Alignment, Application, Command, Element, Error, Event, keyboard, Length, Settings, Subscription, Theme, theme, window};
use iced::widget::{Column, button, Container, text};
use iced::time::{every as iced_time_every};
use iced::window::icon;
use std::time::{Duration, Instant};
use std::sync::mpsc::Receiver;
use crate::config::{Config, SwitchRef};
use crate::sound_thread::{SoundThread, SoundThreadRpc, SoundThreadEvent};
use crate::gamepad_thread::{GamepadThread};
use crate::gui::executor::MyExecutor;
use crate::gui::style::ButtonStyleSheet;
use crate::gui::types::{Message, PlayButtonState};

pub struct ApplicationFlags {
    config: Config,
    sound_thread: SoundThread,
    gamepad_thread: GamepadThread,
    sound_thread_event_receiver: Receiver<SoundThreadEvent>,
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
}

impl MyApplication {
    fn before_close(&mut self) {
        let sound_thread = self.sound_thread.take().unwrap();
        let gamepad_thread = self.gamepad_thread.take().unwrap();

        if let Err(err) = sound_thread.stop() {
            eprintln!("Error while stopping SoundThread: {:?}", err);
        }
        if let Err(err) = gamepad_thread.stop() {
            eprintln!("Error while stopping GamepadThread: {:?}", err);
        }
    }

    fn switch_pressed(&self, switch_ref: SwitchRef) {
        if let Err(err) = self.sound_thread_rpc.switch_pressed(switch_ref) {
            eprintln!("Error sending switch_pressed to SoundThread {}", err);
        }
    }
}

impl Application for MyApplication {
    type Executor = MyExecutor;
    type Message = Message;
    type Theme = Theme;
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
        };

        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from(concat!("μSwitch ", env!("CARGO_PKG_VERSION")))
    }

    fn update(&mut self, message: Message) -> Command<Self::Message>{
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
                println!("Close requested");
                self.before_close();
                return window::close();
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
            iced::subscription::events().map(Message::EventOccurred),
            iced_time_every(Duration::from_millis(8)).map(|_| Message::Tick(Instant::now())),
        ])
    }

    fn view(&self) -> Element<Message> {
        let now = self.now;
        let play_buttons = self.play_buttons.iter();

        let mut column = Column::new()
            .padding(20)
            .align_items(Alignment::Center);

        let mut index = 0;
        for play_button in play_buttons {
            let stylesheet = Box::new(ButtonStyleSheet {
                last_played_ago: play_button.last_played_at.map(|ago| now.duration_since(ago)),
            });

            let button = button(text(&play_button.switch_title))
                .width(Length::Fill)
                .style(theme::Button::Custom(stylesheet))
                .on_press(Message::PlayButtonPressed(index));

            column = column.push(
                Container::new(button)
                .width(Length::Fill).padding(5)
            );
            index += 1;
        }

        column.into()
    }
}

fn make_icon() -> icon::Icon {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/microswitch-icon-32-rgba"));
    let bytes = bytes.to_vec();
    icon::from_rgba(bytes, 32, 32).expect("Failed to load window icon")
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

    // this we will handle ourselves so that we can do cleanup (Event::CloseRequested)
    settings.exit_on_close_request = false;

    settings.window.icon = Some(make_icon());

    // this function will call process::exit() unless there was a startup error
    MyApplication::run(settings)
}
