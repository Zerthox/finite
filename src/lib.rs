mod automaton;
mod dfa;
mod nfa;

pub use automaton::{Automaton, AutomatonError};
pub use dfa::DFA;
pub use nfa::NFA;
