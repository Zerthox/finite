use super::{Automaton, AutomatonError};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, hash::Hash};

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
struct State<S, I>
where
	I: Eq + Hash,
{
	accepts: bool,
	transitions: HashMap<I, S>,
}

impl<S, I> State<S, I>
where
	I: Eq + Hash,
{
	pub fn new(accepts: bool, transitions: HashMap<I, S>) -> Self {
		Self {
			accepts,
			transitions,
		}
	}
}

/// A deterministic finite state automaton.
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DFA<S, I>
where
	S: Default + Clone + Eq + Hash + fmt::Debug + fmt::Display,
	I: Default + Eq + Hash,
{
	current: Option<S>,
	states: HashMap<S, State<S, I>>,
}

impl<S, I> DFA<S, I>
where
	S: Default + Clone + Eq + Hash + fmt::Debug + fmt::Display,
	I: Default + Eq + Hash,
{
	/// Creates a new DFA with a given map of states.
	pub fn from_map<M>(states: M) -> Self
	where
		M: Into<HashMap<S, (bool, HashMap<I, S>)>>,
	{
		Self {
			states: states
				.into()
				.into_iter()
				.map(|(state, (accepts, transitions))| {
					(
						state,
						State {
							accepts,
							transitions,
						},
					)
				})
				.collect(),
			..Self::default()
		}
	}

	/// Returns a reference to the requested state or an `AutomatonError::InexistentState` error otherwise.
	fn get_state(&self, id: &S) -> Result<&State<S, I>, AutomatonError<S>> {
		self.states
			.get(id)
			.ok_or_else(|| AutomatonError::InexistentState(id.clone()))
	}

	/// Returns a mutable reference to the requested state or an `AutomatonError::InexistentState` error otherwise.
	fn get_state_mut(&mut self, id: &S) -> Result<&mut State<S, I>, AutomatonError<S>> {
		self.states
			.get_mut(id)
			.ok_or_else(|| AutomatonError::InexistentState(id.clone()))
	}
}

impl<S, I> Automaton<S, I> for DFA<S, I>
where
	S: Default + Clone + Eq + Hash + fmt::Debug + fmt::Display,
	I: Default + Eq + Hash,
{
	type State = S;

	fn new_state(id: S, _accept: bool) -> Self::State {
		id
	}

	fn has_state(&self, id: &S) -> bool {
		self.states.contains_key(id)
	}

	fn add_state(&mut self, id: S, accept: bool) {
		self.states.insert(id, State::new(accept, HashMap::new()));
	}

	fn add_transition(&mut self, prev: S, input: I, next: S) -> Result<(), AutomatonError<S>> {
		if !self.has_state(&next) {
			Err(AutomatonError::InexistentState(next))
		} else {
			let State { transitions, .. } = self.get_state_mut(&prev)?;
			transitions.insert(input, next);
			Ok(())
		}
	}

	fn get_current(&self) -> Option<&S> {
		self.current.as_ref()
	}

	fn set_current(&mut self, id: S) {
		self.current = if self.has_state(&id) { Some(id) } else { None };
	}

	fn accepts(&self) -> bool {
		match &self.current {
			Some(current) => self.get_state(current).unwrap().accepts,
			None => false,
		}
	}

	fn step(&mut self, input: &I) {
		if let Some(current) = &self.current {
			match self.get_state(current).unwrap().transitions.get(input) {
				Some(next) if self.has_state(next) => self.current = Some(next.clone()),
				_ => self.current = None,
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn construct() {
		// construct a simple DFA
		let mut dfa = DFA::<u32, char>::with_state(0, false);
		dfa.add_state(1, true);
		dfa.add_transition(0, 'a', 1).unwrap();

		// check states
		assert!(dfa.has_state(&0), "Initially added state missing");
		assert!(dfa.has_state(&1), "Later added state missing");
		assert!(!dfa.accepts(), "Initial state incorrectly accepting");
		assert_eq!(
			Some(&0),
			dfa.get_current(),
			"Initial state not set correctly"
		);
	}

	#[test]
	fn run() {
		// construct a new DFA
		let mut dfa = DFA::<u32, char>::with_state(0, false);
		dfa.add_state(1, true);
		dfa.add_transition(0, 'a', 1).unwrap();
		dfa.add_transition(1, 'a', 1).unwrap();
		dfa.add_transition(1, 'b', 1).unwrap();

		// check state setting
		dfa.set_current(1);
		assert_eq!(
			Some(&1),
			dfa.get_current(),
			"Incorrect state after valid state set"
		);
		dfa.set_current(123);
		assert_eq!(
			None,
			dfa.get_current(),
			"Incorrect state after invalid state set"
		);

		// check execution
		dfa.set_current(0);
		assert!(
			dfa.run(&['a', 'a', 'b']),
			"Incorrect result on accepting run"
		);
		assert_eq!(Some(&0), dfa.get_current(), "Incorrect state after run");
		assert!(
			!dfa.run(&"ba".chars().collect::<Vec<_>>()),
			"Incorrect result on not-accepting run"
		);
	}

	#[test]
	fn deserialize() {
		let yaml = r"{states: {0: {accepts: false, transitions: {a: 0, b: 1}}, 1: [true, {b: 1}]}, current: 0}";
		let mut dfa: DFA<u8, char> = serde_yaml::from_str(yaml).unwrap();
		assert!(dfa.has_state(&0), "Deserialized DFA is missing state 0");
		assert!(
			dfa.run(&"bbb".chars().collect::<Vec<_>>()),
			"Incorrect result after run"
		);
	}
}
