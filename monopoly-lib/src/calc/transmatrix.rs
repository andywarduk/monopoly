use nalgebra::DMatrix;
#[cfg(debug_assertions)]
use nalgebra::{Dim, Matrix, RawStorage};
use std::collections::BTreeMap;
use std::hash::Hash;
use std::iter::Sum;
use strum::EnumCount;

use crate::chance::CHCard;
use crate::commchest::CCCard;
use crate::movereason::MoveReason;
use crate::space::{SPACECOUNT, SPACES, Space};
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

        // Calculate steady state matrix
        let steady = Self::calc_steady(&combinedmat, &states, accuracydp, debug);

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

    pub fn steady_group_sum<T, F>(&self, cb: F) -> (Vec<T>, DMatrix<f64>)
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

    pub fn steady_sum<F>(&self, cb: F) -> f64
    where
        F: Fn(&State) -> bool,
    {
        self.steady
            .iter()
            .zip(self.states.keys())
            .filter_map(|(prob, state)| if cb(state) { Some(*prob) } else { None })
            .sum()
    }

    pub fn sum_combined_prob(&self, from_filter: fn(&State) -> bool, to_filter: fn(&State) -> bool) -> Probability {
        Self::sum_matrix_entries(
            self.combinedmat(),
            self.states.keys(),
            self.states.keys(),
            from_filter,
            to_filter,
        )
    }

    pub fn sum_matrix_entries<T: Sum + Copy, RI, CI, RF, CF>(
        matrix: &DMatrix<T>,
        row_iter: impl Iterator<Item = RI>,
        col_iter: impl Iterator<Item = CI> + Clone,
        row_filter: RF,
        col_filter: CF,
    ) -> T
    where
        RF: Fn(RI) -> bool,
        CF: Fn(CI) -> bool,
    {
        matrix
            .row_iter()
            .zip(row_iter)
            .filter_map(|(row, ri)| {
                if row_filter(ri) {
                    Some(
                        row.iter()
                            .zip(col_iter.clone())
                            .filter_map(|(ent, ci)| if col_filter(ci) { Some(*ent) } else { None })
                            .sum::<T>(),
                    )
                } else {
                    None
                }
            })
            .sum()
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
                    print!(
                        "  {start} + {d1}{d2} ({sum}, {}) =>",
                        if double { "double" } else { "not double" }
                    );
                }

                // Calculate state after rolling the dice
                let move_state = if SPACES[start.position] == Space::GoToJail {
                    // In jail
                    match strategy {
                        Strategy::JailWait => {
                            // Wait in jail
                            if double {
                                // Rolled a double, move from just visting, do not get another go
                                State::new(0, Space::find(Space::Visit) + sum as usize, 0)
                            } else {
                                // Did not roll a double
                                let jailrolls = start.jailroll + 1;

                                if jailrolls == 3 {
                                    // Rolled 3 times - move to just visiting
                                    State::new(0, Space::find(Space::Visit), 0)
                                } else {
                                    State::new(0, Space::find(Space::GoToJail), jailrolls)
                                }
                            }
                        }
                        Strategy::PayJail => {
                            // Pay to get out of jail
                            State::new(if double { 1 } else { 0 }, Space::find(Space::Visit) + sum as usize, 0)
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

                if debug {
                    print!(" {}×{}", move_state, ROLL_PROB);
                }

                // Process jumps
                let mut jump_state = JumpState {
                    i,
                    states,
                    jumpmat,
                    combmat: &mut combmat,
                    debug,
                    first: true,
                };

                Self::process_jumps(&mut jump_state, move_state, ROLL_PROB);

                if debug {
                    println!(" )");
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

    fn process_jumps(jump_state: &mut JumpState, move_state: State, parent_prob: Probability) {
        // Get jumps from the new position
        let jumps = jump_state.jumpmat.row(move_state.position);

        // Loop all possible jumps
        for (pos, prob) in jumps.iter().enumerate() {
            if *prob == Probability::NEVER {
                continue;
            }

            let prob = *prob * parent_prob;

            let doubles = if SPACES[pos] == Space::GoToJail {
                0
            } else {
                move_state.doubles
            };

            let new_state = State::new(doubles, pos, move_state.jailroll);

            if pos == move_state.position {
                if jump_state.debug {
                    if jump_state.first {
                        print!(" × ( {}×{}", new_state, prob / ROLL_PROB);
                        jump_state.first = false;
                    } else {
                        print!(" + {}×{}", new_state, prob / ROLL_PROB);
                    }
                }

                // Get matrix column number
                let j = *jump_state.states.get(&new_state).unwrap();

                // Set combined matrix entry
                jump_state.combmat[(jump_state.i, j)] += prob;
            } else {
                // Recurse
                Self::process_jumps(jump_state, new_state, prob);
            }
        }
    }

    fn calc_steady(
        combinedmat: &DMatrix<Probability>,
        states: &BTreeMap<State, usize>,
        accuracydp: u8,
        debug: bool,
    ) -> DMatrix<f64> {
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
                println!(
                    "Iteration {i}: sum {next_sum} (err {}) max delta {max_delta}",
                    (1.0 - next_sum).abs()
                );
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

    pub fn calc_movereason_probabilty(&self) -> Vec<Vec<f64>> {
        let mut probabilities = vec![vec![]; MoveReason::COUNT];

        let (chprob, ccprob, ch3cc3prob) = self.calc_chance_cc_prob();

        let jail = Space::find(Space::GoToJail);
        let visit = Space::find(Space::Visit);

        probabilities[MoveReason::CCCard as usize] = ccprob;
        probabilities[MoveReason::CHCard as usize] = chprob;
        probabilities[MoveReason::CHCardCCCard as usize] = ch3cc3prob;
        probabilities[MoveReason::GoToJail as usize] = vec![0.0; SPACECOUNT];
        probabilities[MoveReason::TripleDouble as usize] = vec![0.0; SPACECOUNT];
        probabilities[MoveReason::NoDouble as usize] = vec![0.0; SPACECOUNT];
        probabilities[MoveReason::ExitJail as usize] = vec![0.0; SPACECOUNT];

        // TODO calculate
        probabilities[MoveReason::TripleDouble as usize][jail] = 0.0; // Prob rolling double 3 times while not in jail
        probabilities[MoveReason::NoDouble as usize][jail] = 0.0; // Prob not rolling double while in jail
        probabilities[MoveReason::GoToJail as usize][jail] = 0.0; // Prob landing on go to jail
        probabilities[MoveReason::ExitJail as usize][visit] = 0.0; // Prob not rolling double 3 times

        probabilities
    }

    fn calc_chance_cc_prob(&self) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let (_, mat) = self.steady_group_sum(|state| Some(state.position));

        // Calculate probability of first landing on a space
        let land1 = mat.map_with_location(|_i, j, p| p / self.jumpmat()[(j, j)].as_f64());

        let mut chprob = vec![0.0; SPACECOUNT];
        let mut ccprob = vec![0.0; SPACECOUNT];
        let mut ch3cc3prob = vec![0.0; SPACECOUNT];

        let ch3pos = Space::find(Space::Chance(2));
        let cc3pos = Space::find(Space::CommunityChest(2));
        let mut ch3cc3probmult = 0.0;

        for (pos, p1) in land1.into_iter().enumerate() {
            match SPACES[pos] {
                Space::CommunityChest(_) => {
                    for (j, p2) in self.jumpmat().row(pos).iter().enumerate().filter(|(j, _)| *j != pos) {
                        ccprob[j] += p1 * p2.as_f64();

                        if pos == cc3pos {
                            ch3cc3prob[j] += p2.as_f64();
                        }
                    }
                }
                Space::Chance(_) => {
                    for (j, p2) in self.jumpmat().row(pos).iter().enumerate().filter(|(j, _)| *j != pos) {
                        chprob[j] += p1 * p2.as_f64();

                        if pos == ch3pos {
                            ch3cc3probmult = p1 * p2.as_f64();
                        }
                    }
                }
                _ => (),
            }
        }

        ch3cc3prob.iter_mut().enumerate().for_each(|(i, p)| {
            *p *= ch3cc3probmult;
            ccprob[i] -= *p;
        });

        chprob[cc3pos] -= ch3cc3prob.iter().sum::<f64>();

        (chprob, ccprob, ch3cc3prob)
    }
}

struct JumpState<'a> {
    i: usize,                              // Matrix row (from)
    states: &'a BTreeMap<State, usize>,    // State map
    jumpmat: &'a DMatrix<Probability>,     // Jump matrix
    combmat: &'a mut DMatrix<Probability>, // Combined matrix
    debug: bool,
    first: bool,
}

#[cfg(debug_assertions)]
fn check_jump_probs(jump_probs: &[(Space, Probability)]) {
    // Sum of probabilities should be 1
    assert_eq!(
        jump_probs.iter().map(|(_, p)| p).copied().sum::<Probability>(),
        Probability::ALWAYS
    );
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
