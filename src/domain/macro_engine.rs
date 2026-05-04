use std::time::{Duration, Instant};

use crate::domain::{PadState, BUTTON_Y};

#[derive(Debug, Clone)]
pub struct MacroEngine {
    looping: bool,
    y_was_down: bool,
    saw_disconnect: bool,
    interval: Duration,
    pulse_width: Duration,
    next_pulse_at: Option<Instant>,
}

impl MacroEngine {
    pub fn new(interval: Duration, pulse_width: Duration) -> Self {
        Self {
            looping: false,
            y_was_down: false,
            saw_disconnect: true,
            interval,
            pulse_width,
            next_pulse_at: None,
        }
    }

    pub fn is_looping(&self) -> bool {
        self.looping
    }

    pub fn process_frame(&mut self, now: Instant, connected: bool, physical: PadState) -> PadState {
        if !connected {
            self.looping = false;
            self.y_was_down = false;
            self.saw_disconnect = true;
            self.next_pulse_at = None;
            return PadState::neutral();
        }

        let y_down = physical.buttons & BUTTON_Y != 0;
        if self.saw_disconnect {
            self.y_was_down = y_down;
            self.saw_disconnect = false;
        } else if y_down && !self.y_was_down {
            self.looping = !self.looping;
            self.next_pulse_at = if self.looping { Some(now) } else { None };
        }
        self.y_was_down = y_down;

        let mut output = physical.without_buttons(BUTTON_Y);
        if !self.looping {
            return output;
        }

        output.left_trigger = 0;

        let next = self.next_pulse_at.get_or_insert(now);
        while now >= *next + self.interval {
            *next += self.interval;
        }

        if now >= *next && now < *next + self.pulse_width {
            output.buttons |= BUTTON_Y;
            output.left_trigger = 255;
        }

        output
    }
}
