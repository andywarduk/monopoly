use monopoly_lib::{
    calc::transmatrix::TransMatrix,
    movereason::MoveReason,
    sim::Board,
    space::{SPACECOUNT, SPACES, Space},
    strategy::Strategy,
};
use strum::IntoEnumIterator;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmBoard {
    board: Board,
}

#[wasm_bindgen]
pub struct WasmStats {
    pub turns: u64,
    pub moves: u64,
    #[wasm_bindgen(getter_with_clone)]
    pub doubles: Vec<u64>,
    #[wasm_bindgen(getter_with_clone)]
    pub rollfreq: Vec<u64>,
    #[wasm_bindgen(getter_with_clone)]
    pub arrivals: Vec<u64>,
    pub reasons_stride: usize,
    #[wasm_bindgen(getter_with_clone)]
    pub reasons: Vec<u64>,
    pub jailwait: bool,
}

#[wasm_bindgen]
impl WasmBoard {
    /// Run the game
    pub fn run(&mut self, ticks: usize) -> WasmStats {
        // Run the requested number of ticks
        for _ in 0..ticks {
            self.board.turn();
        }

        // Return stats
        WasmStats {
            turns: self.board.turns(),
            moves: self.board.moves(),
            doubles: self.board.doubles().to_vec(),
            rollfreq: self.board.rollfreq().to_vec(),
            arrivals: self.board.arrivals().to_vec(),
            reasons_stride: self.board.arrival_reasons()[0].len(),
            reasons: self
                .board
                .arrival_reasons()
                .iter()
                .flat_map(|arr| arr.iter().copied())
                .collect(),
            jailwait: self.board.strategy() == Strategy::JailWait,
        }
    }

    /// Get spaces
    pub fn get_spaces(&self) -> Vec<String> {
        SPACES
            .iter()
            .map(|s| match s {
                Space::Go => "G".to_string(),
                Space::Visit => "J".to_string(),
                Space::FreeParking => "F".to_string(),
                Space::GoToJail => "g".to_string(),
                Space::Property(set, n) => format!("P{}{}", (*set + b'A') as char, n + 1),
                Space::Rail(n) => format!("R{}", n + 1),
                Space::Utility(n) => format!("U{}", n + 1),
                Space::CommunityChest(n) => format!("C{}", n + 1),
                Space::Chance(n) => format!("c{}", n + 1),
                Space::Tax(n) => format!("T{}", n + 1),
            })
            .collect()
    }

    /// Get number of turns performed
    pub fn get_turns(&self) -> u64 {
        self.board.turns()
    }

    /// Get arrival reason descriptions
    pub fn get_arrival_reason_descs(&self) -> Vec<String> {
        MoveReason::iter()
            .filter_map(|r| if r as isize >= 0 { Some(r.to_string()) } else { None })
            .collect()
    }
}

#[wasm_bindgen]
pub fn create_board(jailwait: bool) -> WasmBoard {
    // Create board with requested strategy, cards pulled at random
    WasmBoard {
        board: Board::new(
            if jailwait {
                Strategy::JailWait
            } else {
                Strategy::PayJail
            },
            true,
        ),
    }
}

#[wasm_bindgen]
pub struct WasmFreq {
    #[wasm_bindgen(getter_with_clone)]
    pub space_prob: Vec<f64>,
    #[wasm_bindgen(getter_with_clone)]
    pub movereason_prob: Vec<f64>,
    pub movereason_prob_stride: usize,
}

#[wasm_bindgen]
pub fn get_expected_frequencies(jailwait: bool) -> WasmFreq {
    // Build probability matrices
    let transmatrix = TransMatrix::new(
        if jailwait {
            Strategy::JailWait
        } else {
            Strategy::PayJail
        },
        6,
        false,
    );

    // Sum the steady state matrix by position
    let (_, mat) = transmatrix.steady_group_sum_split(|state| Some(state.position));

    // Convert to vector
    let space_prob = mat.into_iter().copied().collect();

    // Calculate move reason frequencies
    let movereason_prob = transmatrix
        .calc_movereason_probabilty()
        .transpose()
        .iter()
        .copied()
        .collect::<Vec<_>>();

    WasmFreq {
        space_prob,
        movereason_prob,
        movereason_prob_stride: SPACECOUNT,
    }
}
