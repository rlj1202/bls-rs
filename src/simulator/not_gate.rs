const TIME_RAISE: f32 = 0.5;
const TIME_FALL: f32 = 0.5;
const TIME_RANDOM: f32 = 0.5;

#[derive(Clone)]
pub struct NotGate {
    pub wire_in_id: i32,
    pub wire_out_id: i32,

    /// Real state of the output of the gate.
    pub state: bool,

    pub x: u32,
    pub y: u32,
    pub dir: u32,

    /// This value is used to delay changes of the state, mimic the real world thingy.
    pub slow_state: f32,
}

impl NotGate {
    pub fn new(wire_in_id: i32, wire_out_id: i32, x: u32, y: u32, dir: u32) -> Self {
        Self {
            wire_in_id,
            wire_out_id,
            state: false,
            slow_state: 0.0,
            x,
            y,
            dir,
        }
    }

    pub fn update_state(&mut self, state: bool) {
        if state {
            if self.state && self.slow_state >= 1.0 {
                return;
            }

            self.slow_state += TIME_RAISE + TIME_RANDOM * rand::random::<f32>();

            if self.slow_state >= 1.0 {
                self.slow_state = 1.0;
                self.state = true;
            }
        } else {
            if !self.state && self.slow_state <= 0.0 {
                return;
            }

            self.slow_state -= TIME_FALL + TIME_RANDOM * rand::random::<f32>();

            if self.slow_state <= 0.0 {
                self.slow_state = 0.0;
                self.state = false;
            }
        }
    }
}
