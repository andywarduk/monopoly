use monopoly_lib::Board;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmBoard {
    board: Board,
}

#[wasm_bindgen]
pub struct WasmStats {
    pub turns: u64,
    pub moves: u64,
    pub throws: u64,
}

#[wasm_bindgen]
impl WasmBoard {
    #[allow(unused)]
    pub fn run(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.board.turn();
        }
    }

    pub fn get_squares_desc(&self) -> Vec<String> {
        self.board
            .spaces()
            .iter()
            .map(|s| match s {
                monopoly_lib::Space::Go => "Go".to_string(),
                monopoly_lib::Space::Jail => "Jail".to_string(),
                monopoly_lib::Space::FreeParking => "Free Parking".to_string(),
                monopoly_lib::Space::GoToJail => "Go to Jail".to_string(),
                monopoly_lib::Space::Property(set, n) => {
                    format!("{}{}", (*set + b'A') as char, n + 1)
                }
                monopoly_lib::Space::Rail(n) => format!("Rail {}", n + 1),
                monopoly_lib::Space::Utility(n) => match n {
                    0 => "Electric Company".to_string(),
                    1 => "Water Works".to_string(),
                    _ => panic!("Unexpected utility"),
                },
                monopoly_lib::Space::CommunityChest(_) => "Community Chest".to_string(),
                monopoly_lib::Space::Chance(_) => "Chance".to_string(),
                monopoly_lib::Space::Tax(n) => match n {
                    0 => "Income Tax".to_string(),
                    1 => "Luxury Tax".to_string(),
                    _ => panic!("Unexpected tax"),
                },
            })
            .collect()
    }

    pub fn get_squares_type(&self) -> Vec<String> {
        self.board
            .spaces()
            .iter()
            .map(|s| match s {
                monopoly_lib::Space::Go => 'G',
                monopoly_lib::Space::Jail => 'J',
                monopoly_lib::Space::FreeParking => 'F',
                monopoly_lib::Space::GoToJail => 'g',
                monopoly_lib::Space::Property(_, _) => 'P',
                monopoly_lib::Space::Rail(_) => 'R',
                monopoly_lib::Space::Utility(_) => 'U',
                monopoly_lib::Space::CommunityChest(_) => 'C',
                monopoly_lib::Space::Chance(_) => 'c',
                monopoly_lib::Space::Tax(_) => 'T',
            })
            .map(|c| c.to_string())
            .collect()
    }

    pub fn get_stats(&self) -> WasmStats {
        WasmStats {
            turns: self.board.turns(),
            moves: self.board.moves(),
            throws: self.board.throws(),
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
}

#[wasm_bindgen]
pub fn create_board() -> WasmBoard {
    //    alert("in create");
    WasmBoard {
        board: Board::default(),
    }
}
