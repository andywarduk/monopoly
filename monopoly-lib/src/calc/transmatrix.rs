use nalgebra::{Const, DMatrix, DVector, Dyn, OMatrix, SMatrix};
#[cfg(debug_assertions)]
use nalgebra::{Dim, Matrix, RawStorage};
use std::collections::BTreeMap;
use std::hash::Hash;

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
    /// Strategy used for calculation
    strategy: Strategy,
    /// List of states used in the Markov chains
    states: BTreeMap<State, usize>,
    /// Transition matrix of movement depending on dice roll
    movemat: DMatrix<Probability>,
    /// Transition matrix of jumps after landing on a space
    jumpmat: DMatrix<Probability>,
    /// Combined movement and jump transition matrix
    combinedmat: DMatrix<Probability>,
    /// Combined transition matrix steady state vector
    steady: OMatrix<f64, Const<1>, Dyn>,
}

impl TransMatrix {
    /// Calculates transition matrices and steady state (to required dp accuracy) for a given strategy
    pub fn new(strategy: Strategy, accuracydp: u8, debug: bool) -> Self {
        // Build jump matrix
        let jumpmat = Self::build_jumpmat(debug);

        // Create all possible states
        let states = State::create_states(&strategy);

        // Create move matrix
        let (movemat, combinedmat) = Self::build_movemat(&states, &jumpmat, strategy, debug);

        // Calculate steady state vector
        let steady = Self::calc_steady(&combinedmat, accuracydp, debug);

        Self {
            strategy,
            states,
            movemat,
            jumpmat,
            combinedmat,
            steady,
        }
    }

    /// Returns a reference to the state map
    pub fn states(&self) -> &BTreeMap<State, usize> {
        &self.states
    }

    /// Returns a reference to the movement transition matrix
    pub fn movemat(&self) -> &DMatrix<Probability> {
        &self.movemat
    }

    /// Returns a reference to the jump transition matrix
    pub fn jumpmat(&self) -> &DMatrix<Probability> {
        &self.jumpmat
    }

    /// Returns a reference to the combined movement and jump transition matrix
    pub fn combinedmat(&self) -> &DMatrix<Probability> {
        &self.combinedmat
    }

    /// Returns a reference to the combined steady state matrix
    pub fn steady(&self) -> &OMatrix<f64, Const<1>, Dyn> {
        &self.steady
    }

    /// Filters and summarises the steady state matrix to a btree
    pub fn steady_group_sum<T, F>(&self, cb: F) -> BTreeMap<T, f64>
    where
        T: Hash + Eq + Ord,
        F: Fn(&State) -> Option<T>,
    {
        let mut summary = BTreeMap::new();

        // Loop all entries in the steady state vector
        for (prob, state) in self.steady.iter().zip(self.states.keys()) {
            // Call callback to get the group this entry belongs to (if any)
            if let Some(group) = cb(state) {
                // Add to the group
                *summary.entry(group).or_insert(0.0) += *prob;
            }
        }

        summary
    }

    /// Filters and summarises the steady state matrix to a group vector and value matrix
    pub fn steady_group_sum_split<T, F>(&self, cb: F) -> (Vec<T>, DMatrix<f64>)
    where
        T: Clone + Hash + Eq + Ord,
        F: Fn(&State) -> Option<T>,
    {
        // Build summary
        let summary = self.steady_group_sum(cb);

        // Extract group vector
        let groups = summary.keys().cloned().collect::<Vec<_>>();

        // Extract probability vector
        let mat = DMatrix::from_iterator(1, summary.len(), summary.values().cloned());

        (groups, mat)
    }

    /// Get steady state vector entry by state
    pub fn steady_ent(&self, state: &State) -> f64 {
        let ent = self.states.get(state).expect("State not found");
        self.steady[(0, *ent)]
    }

    /// Sum entries in the steady state vector
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

    /// Build the jump transition matrix
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
                print!("  Jump from {startpos}:");
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
                    print!(" {pos}-{probability}");
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

    /// Build the move and combined transition matrices
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

    /// Calculate probability of a jump recursively
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

    /// Calculate the steady state vector from the combined transition matrix
    fn calc_steady(combinedmat: &DMatrix<Probability>, accuracydp: u8, debug: bool) -> OMatrix<f64, Const<1>, Dyn> {
        // Convert combined matrix to floating point
        let combflt = combinedmat.map(|p| p.as_f64());

        // Calculate the required accuracy
        let epsilon = 10f64.powi(-(accuracydp as i32));

        // Set up the system of linear equations

        // Transpose the combined matrix (columns all sum to 1.0)
        let a = combflt.transpose();

        // Get row and column count
        let (rows, cols) = (a.nrows(), a.ncols());

        // Subtract the identity matrix
        let a = a - DMatrix::<f64>::identity(rows, cols);

        // Extend by one row filled with 1.0
        let a_ext = a.resize(rows + 1, cols, 1.0);

        // Create the right-hand side vector with the last element as 1
        // This corresponds to the constraint that the sum of the probabilities is 1
        let b = DVector::<f64>::zeros(rows).push(1.0);

        // Solve the system to find the steady state vector
        let steady = match a_ext.svd(true, true).solve(&b, epsilon) {
            Ok(steady_state) => steady_state.transpose(),
            Err(_) => panic!("Unable to solve steady state matrix"),
        };

        if debug {
            println!("Steady state vector ({}):", steady.ncols());
            println!("{}", steady.transpose());
        }

        steady
    }

    /// Calculate the move reason probability matrix
    pub fn calc_movereason_probabilty(&self) -> SMatrix<f64, { MoveReason::uint_count() }, SPACECOUNT> {
        // Initialsie the matrix
        let mut probabilities: SMatrix<f64, { MoveReason::uint_count() }, SPACECOUNT> = SMatrix::zeros();

        // Calculate chance, community chest and CH3->CC3 matrix rows
        let (chprob, ccprob, ch3cc3prob) = self.calc_chance_cc_prob();

        probabilities.set_row(MoveReason::CCCard as usize, &ccprob);
        probabilities.set_row(MoveReason::CHCard as usize, &chprob);
        probabilities.set_row(MoveReason::CHCardCCCard as usize, &ch3cc3prob);

        // Find spaces
        let g2j = Space::find(Space::GoToJail);
        let visit = Space::find(Space::Visit);

        // Probability of not rolling double while in jail
        let nodoubleprob = if self.strategy == Strategy::JailWait {
            (p!(5 / 6).as_f64() * self.steady_ent(&State::new(0, g2j, 0)))
                + (p!(5 / 6).as_f64() * self.steady_ent(&State::new(0, g2j, 1)))
        } else {
            0.0
        };

        probabilities[(MoveReason::NoDouble as usize, g2j)] = nodoubleprob;

        // Probability of not rolling a double 3 times
        let exitjailprob = if self.strategy == Strategy::JailWait {
            p!(5 / 6).as_f64() * self.steady_ent(&State::new(0, g2j, 2))
        } else {
            0.0
        };

        probabilities[(MoveReason::ExitJail as usize, visit)] = exitjailprob;

        // Probability of rolling double 3 times while not in jail
        let tripledouble = p!(1 / 6).as_f64() * self.steady_sum(|state| state.doubles == 2);

        probabilities[(MoveReason::TripleDouble as usize, g2j)] = tripledouble;

        // Probability of landing on go to jail space
        probabilities[(MoveReason::GoToJail as usize, g2j)] =
            self.steady_sum(|state| state.position == g2j) - probabilities.column(g2j).sum();

        probabilities
    }

    // Calculate chance, community chest and CH3->CC3 move reason probability matrix rows
    fn calc_chance_cc_prob(
        &self,
    ) -> (
        SMatrix<f64, 1, SPACECOUNT>,
        SMatrix<f64, 1, SPACECOUNT>,
        SMatrix<f64, 1, SPACECOUNT>,
    ) {
        // Calculate probability of first landing on a space
        let land1: SMatrix<f64, 1, SPACECOUNT> = SMatrix::from_iterator(
            self.steady_group_sum(|state| Some(state.position))
                .iter()
                .map(|(&i, p)| p / self.jumpmat()[(i, i)].as_f64()),
        );

        // Initialise probability vectors
        let mut chprob: SMatrix<f64, 1, SPACECOUNT> = SMatrix::zeros();
        let mut ccprob: SMatrix<f64, 1, SPACECOUNT> = SMatrix::zeros();
        let mut ch3cc3prob: SMatrix<f64, 1, SPACECOUNT> = SMatrix::zeros();

        // Find spaces
        let ch3pos = Space::find(Space::Chance(2));
        let cc3pos = Space::find(Space::CommunityChest(2));

        // Initialise CH3->CC3 probability
        let mut ch3cc3probmult = 0.0;

        for (pos, p1) in land1.iter().enumerate() {
            match SPACES[pos] {
                Space::CommunityChest(_) => {
                    // Find jump probabilities for community chest space (excluding self)
                    for (j, p2) in self.jumpmat().row(pos).iter().enumerate().filter(|(j, _)| *j != pos) {
                        // Calculate the resultant probability (landing on space then jumping)
                        ccprob[j] += p1 * p2.as_f64();

                        if pos == cc3pos {
                            // Save probability of jumping in to CH3 -> CC3 vector as well (multiplied later)
                            ch3cc3prob[j] += p2.as_f64();
                        }
                    }
                }
                Space::Chance(_) => {
                    // Find jump probabilities for chance space (excluding self)
                    for (j, p2) in self.jumpmat().row(pos).iter().enumerate().filter(|(j, _)| *j != pos) {
                        // Calculate the resultant probability (landing on space then jumping)
                        chprob[j] += p1 * p2.as_f64();

                        if pos == ch3pos && j == cc3pos {
                            // Save probability of landing on CH3 then going to CC3
                            ch3cc3probmult = p1 * p2.as_f64();
                        }
                    }
                }
                _ => (),
            }
        }

        // Multiply each CH3 -> CC3 probabilities by the probability of going from CH3 to CC3
        ch3cc3prob.iter_mut().enumerate().for_each(|(i, p)| {
            *p *= ch3cc3probmult;

            // Subtract the resultant probability from the community chest probability
            ccprob[i] -= *p;
        });

        // Subtract the total CH3 -> CC3 probability vector from the CH3 -> CC3 probability
        let ch3cc3tot = ch3cc3prob.iter().sum::<f64>();
        chprob[cc3pos] -= ch3cc3tot;

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
/// Checks the sum of probabilities is 1.0
fn check_jump_probs(jump_probs: &[(Space, Probability)]) {
    assert_eq!(
        jump_probs.iter().map(|(_, p)| p).copied().sum::<Probability>(),
        Probability::ALWAYS
    );
}

#[cfg(debug_assertions)]
/// Checks the sum of probabilities in each matrix row is 1.0
fn check_matrix<R, C, S>(transprob: &Matrix<Probability, R, C, S>)
where
    R: Dim,
    C: Dim,
    S: RawStorage<Probability, R, C>,
{
    for row in transprob.row_iter() {
        assert_eq!(row.iter().copied().sum::<Probability>(), Probability::ALWAYS);
    }
}
