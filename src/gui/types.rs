use std::time::Instant;
use iced::Event;

pub struct PlayButtonState {
    pub switch_title: String,
    /// Is the keyboard key currently down? A sample should only play for a single press, without repeating.
    pub key_held_down: bool,
    /// The last time the corresponding sample (the one specified by config.switches[button_index].play.unwrap().bank_sample_ref) has been played.
    pub last_played_at: Option<Instant>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
    EventOccurred(Event),
    PlayButtonPressed(usize), // (index)
}
