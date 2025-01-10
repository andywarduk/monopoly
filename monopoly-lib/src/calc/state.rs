use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{Display, Formatter, Result},
};

use crate::space::{SPACES, Space};
use crate::strategy::Strategy;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct State {
    pub position: usize,
    pub doubles: u8,
    pub jailroll: u8,
}

impl State {
    pub fn new(doubles: u8, position: usize, jailroll: u8) -> Self {
        Self {
            position,
            doubles,
            jailroll,
        }
    }

    pub fn create_states(strategy: &Strategy) -> BTreeMap<State, usize> {
        let mut states = BTreeSet::new();

        for doubles in 0..3 {
            (0..SPACES.len()).for_each(|position| {
                if SPACES[position] != Space::GoToJail {
                    states.insert(State {
                        doubles,
                        position,
                        jailroll: 0,
                    });
                }
            });
        }

        let jail = Space::find(Space::GoToJail);

        match strategy {
            Strategy::PayJail => {
                states.insert(State {
                    doubles: 0,
                    position: jail,
                    jailroll: 0,
                });

                assert_eq!(states.len(), 118);
            }
            Strategy::JailWait => {
                for jailroll in 0..3 {
                    states.insert(State {
                        doubles: 0,
                        position: jail,
                        jailroll,
                    });
                }

                assert_eq!(states.len(), 120);
            }
        }

        states.into_iter().enumerate().map(|(i, s)| (s, i)).collect()
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut output = format!("[{}", SPACES[self.position].shortdesc());

        if self.doubles > 0 {
            output.push_str(&format!(" d{}", self.doubles));
        }

        if SPACES[self.position] == Space::GoToJail {
            output.push_str(&format!(" r{}", self.jailroll));
        } else {
            assert!(self.jailroll == 0);
        }

        output.push(']');

        output.fmt(f)
    }
}
