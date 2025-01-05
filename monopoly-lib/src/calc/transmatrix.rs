use nalgebra::DMatrix;
#[cfg(debug_assertions)]
use nalgebra::{Dim, Matrix, RawStorage};
use std::collections::BTreeMap;
use std::hash::Hash;

use crate::chance::CHCard;
use crate::commchest::CCCard;
use crate::space::{SPACES, Space};
use crate::strategy::Strategy;

use super::dice::dice_rolls;
use super::probability::{Probability, p};
use super::state::State;

const ROLL_PROB: Probability = p!(1 / 36);

pub struct TransMatrix {
    states: BTreeMap<State, usize>,
    movemat: DMatrix<Probability>,
    jumpmat: DMatrix<Probability>,
    combinedmat: DMatrix<Probability>,
    steady: DMatrix<f64>,
}

impl TransMatrix {
    pub fn new(strategy: Strategy, accuracydp: u8, debug: bool) -> Self {
        // Build jump matrix
        let jumpmat = Self::build_jumpmat(debug);

        // Create all possible states
        let states = State::create_states(&strategy);

        // Create move matrix
        let (movemat, combinedmat) = Self::build_movemat(&states, &jumpmat, strategy, debug);

        // Create steady state matrix
        let steady = Self::build_steady(&combinedmat, &states, accuracydp, debug);

        Self {
            states,
            movemat,
            jumpmat,
            combinedmat,
            steady,
        }
    }

    pub fn states(&self) -> &BTreeMap<State, usize> {
        &self.states
    }

    pub fn movemat(&self) -> &DMatrix<Probability> {
        &self.movemat
    }

    pub fn jumpmat(&self) -> &DMatrix<Probability> {
        &self.jumpmat
    }

    pub fn combinedmat(&self) -> &DMatrix<Probability> {
        &self.combinedmat
    }

    pub fn steady(&self) -> &DMatrix<f64> {
        &self.steady
    }

    pub fn steady_summary<T, F>(&self, cb: F) -> (Vec<T>, DMatrix<f64>)
    where
        T: Clone + Hash + Eq + Ord,
        F: Fn(&State) -> Option<T>,
    {
        let mut summary = BTreeMap::new();

        for (prob, state) in self.steady.iter().zip(self.states.keys()) {
            if let Some(group) = cb(state) {
                *summary.entry(group).or_insert(0.0) += *prob;
            }
        }

        let groups = summary.keys().cloned().collect::<Vec<_>>();
        let mat = DMatrix::from_iterator(1, summary.len(), summary.values().cloned());

        (groups, mat)
    }

    fn build_jumpmat(debug: bool) -> DMatrix<Probability> {
        // Initialise jump transition map
        let dim = SPACES.len();
        let mut jumpmat = DMatrix::from_element(dim, dim, Probability::NEVER);

        let ccdeck = CCCard::build_deck();
        let ccprob = p!(1, ccdeck.len() as u64);
        let chdeck = CHCard::build_deck();
        let chprob = p!(1, chdeck.len() as u64);

        // Loop all positions and build jump probability map
        for (startidx, startpos) in SPACES.iter().enumerate() {
            if debug {
                print!("  Jump from {}:", startpos.shortdesc());
            }

            // Handle Go to jail / Chance / Community chest jumps
            let jump_probs = match startpos {
                Space::CommunityChest(_) => {
                    // Community chest
                    ccdeck
                        .iter()
                        .map(|card| match card {
                            CCCard::GoGo => (Space::Go, ccprob),            // Go to Go
                            CCCard::GoJail => (Space::GoToJail, ccprob),    // Go to Jail
                            CCCard::Inconsequential => (*startpos, ccprob), // Stay on community chest
                        })
                        .collect()
                }
                Space::Chance(_) => {
                    // Chance
                    chdeck
                        .iter()
                        .map(|card| match card {
                            CHCard::GoGo => (Space::Go, chprob),                                // Go to Go
                            CHCard::GoJail => (Space::GoToJail, chprob),                        // Go to Jail
                            CHCard::GoProperty(set, i) => (Space::Property(*set, *i), chprob),  // Go to property
                            CHCard::GoRail(i) => (Space::Rail(*i), chprob),                     // Go to Rail
                            CHCard::GoNextRail => (SPACES[Space::next_rail(startidx)], chprob), // Go to next rail
                            CHCard::GoNextUtil => (SPACES[Space::next_util(startidx)], chprob), // Go to next utility
                            CHCard::Back3 => (SPACES[startidx - 3], chprob),                    // Go back 3
                            CHCard::Inconsequential => (*startpos, chprob),                     // Stay on chance (6/16)
                        })
                        .collect()
                }
                Space::GoToJail => {
                    // Go to jail
                    vec![(Space::GoToJail, Probability::ALWAYS)]
                }
                _ => vec![(*startpos, Probability::ALWAYS)],
            };

            #[cfg(debug_assertions)]
            check_jump_probs(&jump_probs);

            for (pos, probability) in &jump_probs {
                let i = startidx;
                let j = Space::find(*pos);

                jumpmat[(i, j)] += *probability;

                if debug {
                    print!(" {}-{probability}", pos.shortdesc());
                }
            }

            if debug {
                println!();
            }
        }

        #[cfg(debug_assertions)]
        check_matrix(&jumpmat);

        jumpmat
    }

    fn build_movemat(
        states: &BTreeMap<State, usize>,
        jumpmat: &DMatrix<Probability>,
        strategy: Strategy,
        debug: bool,
    ) -> (DMatrix<Probability>, DMatrix<Probability>) {
        // Initialise transition maps
        let mut movemat = DMatrix::from_element(states.len(), states.len(), Probability::NEVER);
        let mut combmat = DMatrix::from_element(states.len(), states.len(), Probability::NEVER);

        // Loop all start states
        for (start, &i) in states.iter() {
            if debug {
                println!("From {}:", start);
            }

            // For each possible dice roll
            for (d1, d2, sum, double) in dice_rolls() {
                if debug {
                    print!("  {start} + {d1}{d2} ({sum}, {}) -> ", if double { "double" } else { "not double" });
                }

                // Calculate state after rolling the dice
                let move_state = if SPACES[start.position] == Space::GoToJail {
                    // In jail
                    match strategy {
                        Strategy::JailWait => {
                            // Wait in jail
                            if double {
                                // Rolled a double, move from just visting, do not get another go
                                State::new(0, Space::find(Space::Jail) + sum as usize, 0)
                            } else {
                                // Did not roll a double
                                let jailrolls = start.jailroll + 1;

                                if jailrolls == 3 {
                                    // Rolled 3 times - move to just visiting
                                    State::new(0, Space::find(Space::Jail), 0)
                                } else {
                                    State::new(0, Space::find(Space::GoToJail), jailrolls)
                                }
                            }
                        }
                        Strategy::PayJail => {
                            // Pay to get out of jail
                            State::new(if double { 1 } else { 0 }, Space::find(Space::Jail) + sum as usize, 0)
                        }
                    }
                } else {
                    // Normal move
                    let mut doubles = if !double { 0 } else { start.doubles + 1 };

                    if doubles == 3 {
                        // 3 doubles in a row, go to jail
                        State::new(0, Space::find(Space::GoToJail), 0)
                    } else {
                        let position = (start.position + sum as usize) % SPACES.len();

                        if SPACES[position] == Space::GoToJail {
                            // Go to jail
                            doubles = 0;
                        }

                        State::new(doubles, position, 0)
                    }
                };

                // Set move matrix entry
                let j = *states.get(&move_state).unwrap();
                movemat[(i, j)] += ROLL_PROB;

                // Process jumps
                Self::process_jumps(i, move_state, ROLL_PROB, states, jumpmat, &mut combmat, debug);

                if debug {
                    println!();
                }
            }

            #[cfg(debug_assertions)]
            check_matrix(&combmat.row(i));
        }

        #[cfg(debug_assertions)]
        check_matrix(&movemat);

        #[cfg(debug_assertions)]
        check_matrix(&combmat);

        (movemat, combmat)
    }

    fn process_jumps(
        i: usize,
        move_state: State,
        probability: Probability,
        states: &BTreeMap<State, usize>,
        jumpmat: &DMatrix<Probability>,
        combmat: &mut DMatrix<Probability>,
        debug: bool,
    ) {
        // Get jumps from the new position
        let jumps = jumpmat.row(move_state.position);

        // Loop all possible jumps
        for (pos, prob) in jumps.iter().enumerate() {
            if *prob == Probability::NEVER {
                continue;
            }

            let doubles = if SPACES[pos] == Space::GoToJail { 0 } else { move_state.doubles };

            let jump_state = State::new(doubles, pos, move_state.jailroll);

            if pos == move_state.position {
                if debug {
                    print!(" {}-{prob}", jump_state);
                }

                // Set combined matrix entry
                let j = *states.get(&jump_state).unwrap();
                combmat[(i, j)] += probability * *prob;
            } else {
                // Recurse
                Self::process_jumps(i, jump_state, probability * *prob, states, jumpmat, combmat, debug);
            }
        }
    }

    fn build_steady(combinedmat: &DMatrix<Probability>, states: &BTreeMap<State, usize>, accuracydp: u8, debug: bool) -> DMatrix<f64> {
        // Convert combined matrix to floating point
        let combflt = combinedmat.map(|p| p.as_f64());

        // Create steady state matrix for iteration
        let mut steady = DMatrix::from_element(1, states.len(), (Probability::ALWAYS / states.len()).as_f64());

        // Calculate the required accuracy
        let req_acc = 10f64.powi(-(accuracydp as i32));

        // Solved flag
        let mut solved = false;

        for i in 0..250 {
            // Calculate the next state
            let next_steady = &steady * &combflt;

            // Get the sum of the next state (should be ~1.0)
            let next_sum = next_steady.iter().sum::<f64>();

            // Calculate the delta between the current and next state
            let delta = &next_steady - &steady;
            let max_delta = delta.iter().map(|x| x.abs()).fold(0.0, |acc: f64, x| acc.max(x));

            if debug {
                println!("Iteration {i}: sum {next_sum} (err {}) max delta {max_delta}", (1.0 - next_sum).abs());
            }

            // Check sum is ~= 1.0
            #[cfg(debug_assertions)]
            {
                let err = (1.0 - next_sum).abs();
                assert!(err < (100.0 * f64::EPSILON));
            }

            // Set next state as current state
            steady = next_steady;

            if max_delta < req_acc {
                if debug {
                    println!("Required accuracy found");
                }

                solved = true;

                break;
            }
        }

        assert!(solved, "Unable to solve steady state matrix");

        steady
    }
}

#[cfg(debug_assertions)]
fn check_jump_probs(jump_probs: &[(Space, Probability)]) {
    // Sum of probabilities should be 1
    assert_eq!(jump_probs.iter().map(|(_, p)| p).copied().sum::<Probability>(), Probability::ALWAYS);
}

#[cfg(debug_assertions)]
fn check_matrix<R, C, S>(transprob: &Matrix<Probability, R, C, S>)
where
    R: Dim,
    C: Dim,
    S: RawStorage<Probability, R, C>,
{
    // Sum of probabilities for each row should be 1

    for row in transprob.row_iter() {
        assert_eq!(row.iter().copied().sum::<Probability>(), Probability::ALWAYS);
    }
}
