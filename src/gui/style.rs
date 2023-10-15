use std::time::Duration;
use iced::{Background, Color, Theme, Vector};
use iced::widget::button::{StyleSheet, Appearance};

// Whenever a sample is played, the corresponding button will be highlighted temporarily.
// These constant specify the duration and colour of this highlight.
// Buttons that only play a random or a stepped sample, will not be highlighted.
const BUTTON_PLAY_FEEDBACK_DURATION: u128 = 1000;
const BUTTON_PLAY_FEEDBACK_FROM: (f32, f32, f32) = (51.0 / 255.0, 147.0 / 255.0, 129.9 / 255.0);
const BUTTON_PLAY_FEEDBACK_TO: (f32, f32, f32) = (0.87, 0.87, 0.87);

fn interpolate(from: f32, to: f32, progress: f32) -> f32 {
    (to - from) * progress + from
}

pub struct ButtonStyleSheet {
    pub last_played_ago: Option<Duration>,
}

impl StyleSheet for ButtonStyleSheet {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> Appearance {
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

        Appearance {
            shadow_offset: Vector::new(0.0, 0.0),
            background: Some(Background::Color(background_color)),
            border_radius: 2.0.into(),
            border_width: 1.0,
            border_color: [0.7, 0.7, 0.7].into(),
            text_color: Color::BLACK,
        }
    }
}
