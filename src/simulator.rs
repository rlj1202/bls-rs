use bevy::prelude::*;

mod conductive;
mod not_gate;
mod union_find;
mod wire;

use conductive::*;
pub use not_gate::*;
use union_find::*;
pub use wire::*;

pub struct Simulator {
    pub width: u32,
    pub height: u32,

    pub raw_image: Image,

    pub wire_map: Vec<Vec<i32>>,

    pub gates: Vec<NotGate>,
    pub wires: Vec<Wire>,
}

impl Simulator {
    pub fn from_image(image: &Image) -> Self {
        let raw_image = image.clone();

        let image_size = image.texture_descriptor.size;
        let components = image.texture_descriptor.format.describe().components;

        let pixels: Vec<&[u8]> = image.data.chunks_exact(components as usize).collect();
        let rows: Vec<&[&[u8]]> = pixels.chunks_exact(image_size.width as usize).collect();

        // Find wires horizontally
        let mut wire_map = vec![vec![-1i32; image_size.width as usize]; image_size.height as usize];
        let mut wire_last_id: i32 = 0;

        let mut prev_is_conductive = true;
        for y in 0..image_size.height as usize {
            for x in 0..image_size.width as usize {
                let pixel = &rows[y][x];

                let cur_is_conductive = pixel.is_conductive();

                if cur_is_conductive {
                    if !prev_is_conductive {
                        wire_last_id += 1;
                    }

                    wire_map[y][x] = wire_last_id;
                }

                prev_is_conductive = cur_is_conductive;
            }
        }

        // Merge wires vertically using union-find
        let mut wire_merge = vec![-1i32; (wire_last_id + 1) as usize];

        for x in 0..image_size.width as usize {
            for y in 1..image_size.height as usize {
                let prev_wire_id = wire_map[y - 1][x];
                let cur_wire_id = wire_map[y][x];

                if prev_wire_id == -1 || cur_wire_id == -1 {
                    continue;
                }

                wire_merge.merge(prev_wire_id, cur_wire_id);
            }
        }

        // find crossing wires and not gates
        let mut gates: Vec<NotGate> = Vec::new();

        for y in 1..(image_size.height - 1) as usize {
            for x in 1..(image_size.height - 1) as usize {
                if wire_map[y - 1][x] == -1
                    || wire_map[y + 1][x] == -1
                    || wire_map[y][x - 1] == -1
                    || wire_map[y][x + 1] == -1
                    || wire_map[y][x] != -1
                {
                    continue;
                }

                let bl = wire_map[y + 1][x - 1] != -1;
                let br = wire_map[y + 1][x + 1] != -1;
                let tl = wire_map[y - 1][x - 1] != -1;
                let tr = wire_map[y - 1][x + 1] != -1;

                // tl . tr
                // .  .  .
                // bl . br

                match (bl, br, tl, tr) {
                    (false, false, false, false) => {
                        // crossing wire
                        // .#.
                        // #.#
                        // .#.
                        let left_wire_id = wire_map[y][x - 1];
                        let right_wire_id = wire_map[y][x + 1];
                        let top_wire_id = wire_map[y - 1][x];
                        let bottom_wire_id = wire_map[y + 1][x];
                        wire_merge.merge(left_wire_id, right_wire_id);
                        wire_merge.merge(top_wire_id, bottom_wire_id);
                    }
                    (true, true, false, false) => {
                        // not gate up
                        // .#.
                        // #.#
                        // ###
                        let top_wire_id = wire_merge.find(wire_map[y - 1][x]);
                        let bottom_wire_id = wire_merge.find(wire_map[y + 1][x]);
                        gates.push(NotGate::new(
                            bottom_wire_id,
                            top_wire_id,
                            x as u32,
                            y as u32,
                            0,
                        ));
                    }
                    (false, false, true, true) => {
                        // not gate down
                        // ###
                        // #.#
                        // .#.
                        let top_wire_id = wire_merge.find(wire_map[y - 1][x]);
                        let bottom_wire_id = wire_merge.find(wire_map[y + 1][x]);
                        gates.push(NotGate::new(
                            top_wire_id,
                            bottom_wire_id,
                            x as u32,
                            y as u32,
                            0,
                        ));
                    }
                    (false, true, false, true) => {
                        // not gate left
                        // .##
                        // #.#
                        // .##
                        let left_wire_id = wire_merge.find(wire_map[y][x - 1]);
                        let right_wire_id = wire_merge.find(wire_map[y][x + 1]);
                        gates.push(NotGate::new(
                            right_wire_id,
                            left_wire_id,
                            x as u32,
                            y as u32,
                            0,
                        ));
                    }
                    (true, false, true, false) => {
                        // not gate right
                        // ##.
                        // #.#
                        // ##.
                        let left_wire_id = wire_merge.find(wire_map[y][x - 1]);
                        let right_wire_id = wire_merge.find(wire_map[y][x + 1]);
                        gates.push(NotGate::new(
                            left_wire_id,
                            right_wire_id,
                            x as u32,
                            y as u32,
                            0,
                        ));
                    }
                    _ => (),
                }
            }
        }

        // compress wire id
        let mut wire_remap = vec![0i32; (wire_last_id + 1) as usize];
        let mut wires: Vec<Wire> = Vec::new();

        for wire_id in 0..(wire_last_id + 1) {
            if !wire_merge.is_root(wire_id) {
                continue;
            }

            wire_remap[wire_id as usize] = wires.len() as i32;
            wires.push(Wire::new());
        }

        for x in 0..image_size.width as usize {
            for y in 0..image_size.height as usize {
                let wire_id = wire_merge.find(wire_map[y][x]);
                if wire_id == -1 {
                    continue;
                }

                let wire_id = wire_remap[wire_id as usize];
                wire_map[y][x] = wire_id;
            }
        }

        for gate in &mut gates {
            gate.wire_in_id = wire_remap[wire_merge.find(gate.wire_in_id) as usize];
            gate.wire_out_id = wire_remap[wire_merge.find(gate.wire_out_id) as usize];
        }

        // connect gates
        for (gate_id, gate) in gates.iter().enumerate() {
            wires[gate.wire_out_id as usize]
                .gate_in_ids
                .push(gate_id as u32);
            wires[gate.wire_in_id as usize]
                .gate_out_ids
                .push(gate_id as u32);
        }

        Self {
            width: image_size.width,
            height: image_size.height,
            raw_image,
            wire_map,
            gates,
            wires,
        }
    }

    /// Set state of the wire the coordinates points to.
    /// If there is no wire the coordinates points to, returns false.
    /// Otherwise returns true.
    pub fn set(&mut self, x: u32, y: u32, state: bool) -> bool {
        let wire_id = self.wire_map[y as usize][x as usize];

        if wire_id == -1 {
            return false;
        }

        self.wires[wire_id as usize].state = state;

        return true;
    }

    /// Get state of the wire.
    /// If there are some gates connected to the wire, the state of the wire is ON if at least one of the gates is ON.
    /// If there are no gates connected to the wire, the state of the wire can be set by user interaction.
    pub fn get_wire_state(&self, wire_id: u32) -> bool {
        let wire = &self.wires[wire_id as usize];

        if wire.gate_in_ids.is_empty() {
            wire.state
        } else {
            wire.gate_in_ids
                .iter()
                .any(|&gate_id| self.gates[gate_id as usize].state)
        }
    }

    pub fn simulate_one_step(&mut self) {
        for wire_id in 0..self.wires.len() {
            let wire_state = self.get_wire_state(wire_id as u32);
            let wire = &mut self.wires[wire_id];
            wire.state = wire_state;
        }

        for gate in self.gates.iter_mut() {
            let wire_state = self.wires[gate.wire_in_id as usize].state;
            gate.update_state(!wire_state);
        }
    }

    pub fn simulate(&mut self, steps: u32) {
        for _ in 0..steps {
            self.simulate_one_step();
        }
    }
}
