use monopoly_lib::{
    calc::transmatrix::TransMatrix,
    sim::{Board, movereason::MoveReason},
    space::{SPACES, Space},
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
    pub jailwait: bool,
}

#[wasm_bindgen]
impl WasmBoard {
    #[allow(unused)]
    pub fn run(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.board.turn();
        }
    }

    pub fn get_spaces_desc(&self) -> Vec<String> {
        SPACES
            .iter()
            .map(|s| match s {
                Space::Go => "Go".to_string(),
                Space::Visit => "Jail".to_string(),
                Space::FreeParking => "Free Parking".to_string(),
                Space::GoToJail => "Go to Jail".to_string(),
                Space::Property(set, n) => {
                    format!("{}{}", (*set + b'A') as char, n + 1)
                }
                Space::Rail(n) => format!("R{}", n + 1),
                Space::Utility(n) => match n {
                    0 => "Electric Company".to_string(),
                    1 => "Water Works".to_string(),
                    _ => panic!("Unexpected utility"),
                },
                Space::CommunityChest(n) => format!("C{}", n + 1),
                Space::Chance(n) => format!("c{}", n + 1),
                Space::Tax(n) => match n {
                    0 => "Income Tax".to_string(),
                    1 => "Luxury Tax".to_string(),
                    _ => panic!("Unexpected tax"),
                },
            })
            .collect()
    }

    pub fn get_spaces_type(&self) -> Vec<String> {
        SPACES
            .iter()
            .map(|s| match s {
                Space::Go => 'G',
                Space::Visit => 'J',
                Space::FreeParking => 'F',
                Space::GoToJail => 'g',
                Space::Property(_, _) => 'P',
                Space::Rail(_) => 'R',
                Space::Utility(n) => match n {
                    0 => 'U',
                    1 => 'u',
                    _ => panic!("unrecognised utility"),
                },
                Space::CommunityChest(_) => 'C',
                Space::Chance(_) => 'c',
                Space::Tax(n) => match n {
                    0 => 'T',
                    1 => 't',
                    _ => panic!("unrecognised tax"),
                },
            })
            .map(|c| c.to_string())
            .collect()
    }

    pub fn get_stats(&self) -> WasmStats {
        WasmStats {
            turns: self.board.turns(),
            moves: self.board.moves(),
            jailwait: self.board.strategy() == Strategy::JailWait,
        }
    }

    pub fn get_doubles(&self) -> Vec<u64> {
        self.board.doubles().to_vec()
    }

    pub fn get_arrivals(&self) -> Vec<u64> {
        self.board.arrivals().to_vec()
    }

    pub fn get_arrival_reasons(&self, elem: usize) -> Vec<u64> {
        self.board.arrival_reasons(elem).to_vec()
    }

    pub fn get_arrival_reason_descs(&self) -> Vec<String> {
        MoveReason::iter()
            .filter_map(|r| if r.clone() as isize >= 0 { Some(r.to_string()) } else { None })
            .collect()
    }

    pub fn get_rollfreq(&self) -> Vec<u64> {
        self.board.rollfreq().to_vec()
    }
}

#[wasm_bindgen]
pub fn create_board(jailwait: bool) -> WasmBoard {
    // Create board with requested strategy, cards pulled at random
    WasmBoard {
        board: Board::new(if jailwait { Strategy::JailWait } else { Strategy::PayJail }, true),
    }
}

#[wasm_bindgen]
pub fn get_expected_frequencies(jailwait: bool) -> Vec<f64> {
    // Build probability matrices
    let transmatrix = TransMatrix::new(if jailwait { Strategy::JailWait } else { Strategy::PayJail }, 6, false);

    // Get the steady state matrix
    let (_, mat) = transmatrix.steady_summary(|state| Some(state.position));

    // Convert to vector
    mat.into_iter().copied().collect()
}
