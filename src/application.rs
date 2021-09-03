use crate::config::{Config, SwitchRef};
use iced::{button, Button, Column, Text, Settings, Error, Element, Align, Length, Container, Application, executor, Clipboard, Command, Subscription};
use iced_native::{Event, keyboard, window};
use iced::window::Icon;
use crate::sound_thread::{SoundThread, SoundThreadRpc};
use crate::gamepad_thread::GamepadThread;

pub struct ApplicationFlags {
    config: Config,
    sound_thread: SoundThread,
    gamepad_thread: GamepadThread,
}

struct PlayButtonState {
    switch_title: String,
    button: button::State,
    pressed: bool,
}

pub struct MyApplication {
    config: Config,
    sound_thread: Option<SoundThread>,
    sound_thread_rpc: SoundThreadRpc,
    gamepad_thread: Option<GamepadThread>,
    play_buttons: Vec<PlayButtonState>,
    should_exit: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
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
        let switches = &config.switches;

        let play_buttons = switches
            .into_iter()
            .map(|switch_config| PlayButtonState {
                switch_title: switch_config.title.clone(),
                button: button::State::new(),
                pressed: false,
            })
            .collect();

        let app = MyApplication {
            config,
            sound_thread: Some(sound_thread),
            sound_thread_rpc,
            gamepad_thread: Some(gamepad_thread),
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
                    let was_pressed = button_state.pressed;
                    button_state.pressed = true;

                    if !was_pressed {
                        self.switch_pressed(switch_ref);
                    }
                }
            },
            Message::EventOccurred(Event::Keyboard(keyboard::Event::KeyReleased { key_code, modifiers: _ })) => {
                println!("Keyboard release {:?}", key_code);

                if let Some(switch_config) = self.config.find_switch_for_keyboard_key(key_code) {
                    let button_state = &mut self.play_buttons[switch_config.switch_ref.switch_index];
                    button_state.pressed = false;
                }
            },
            _ => {},
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::EventOccurred)
    }

    fn view(&mut self) -> Element<Message> {
        let play_buttons = self.play_buttons.iter_mut();

        let mut column = Column::new()
            .padding(20)
            .align_items(Align::Center);

        let mut index = 0;
        for play_button in play_buttons {
            let button = Button::new(&mut play_button.button, Text::new(&play_button.switch_title))
                .width(Length::Fill)
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
) -> Result<(), Error> {
    let config = config.clone();
    let flags = ApplicationFlags { config, sound_thread, gamepad_thread };
    let mut settings = Settings::with_flags(flags);

    // this we will handle ourselves so that we can do cleanup
    settings.exit_on_close_request = false;

    settings.window.icon = Some(make_icon());

    // this function will call process::exit() unless there was a startup error
    MyApplication::run(settings)
}
