pub struct Wire {
    pub gate_in_ids: Vec<u32>,
    pub gate_out_ids: Vec<u32>,

    pub state: bool,
}

impl Default for Wire {
    fn default() -> Self {
        Self {
            gate_in_ids: Vec::new(),
            gate_out_ids: Vec::new(),
            state: false,
        }
    }
}

impl Wire {
    pub fn new() -> Self {
        Self::default()
    }
}
