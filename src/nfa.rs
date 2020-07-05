use super::{Automaton, AutomatonError};
use serde::{Deserialize, Serialize};
use std::{
	collections::{HashMap, HashSet},
	fmt,
	hash::Hash,
};

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct State<S, I>
where
	S: Eq + Hash,
	I: Eq + Hash,
{
	accepts: bool,
	transitions: HashMap<I, HashSet<S>>,
}

impl<S, I> State<S, I>
where
	S: Eq + Hash,
	I: Eq + Hash,
{
	pub fn new(accepts: bool, transitions: HashMap<I, HashSet<S>>) -> Self {
		Self {
			accepts,
			transitions,
		}
	}
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NFA<S, I>
where
	S: Default + Clone + Eq + Hash + fmt::Debug + fmt::Display,
	I: Default + Eq + Hash,
{
	current: HashSet<S>,
	states: HashMap<S, State<S, I>>,
}

impl<S, I> NFA<S, I>
where
	S: Default + Clone + Eq + Hash + fmt::Debug + fmt::Display,
	I: Default + Eq + Hash,
{
	/// Creates a new NFA with a given map of states.
	pub fn from_map<M>(states: M) -> Self
	where
		M: Into<HashMap<S, (bool, HashMap<I, HashSet<S>>)>>,
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

impl<S, I> Automaton<S, I> for NFA<S, I>
where
	S: Default + Clone + Eq + Hash + fmt::Debug + fmt::Display,
	I: Default + Eq + Hash,
{
	type State = HashSet<S>;

	fn new_state(id: S, _accept: bool) -> Self::State {
		let mut state = HashSet::with_capacity(1);
		state.insert(id);
		state
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
			if let Some(set) = transitions.get_mut(&input) {
				set.insert(next);
			} else {
				transitions.insert(input, Self::new_state(next, true));
			}
			Ok(())
		}
	}

	fn get_current(&self) -> Option<&Self::State> {
		if !self.current.is_empty() {
			Some(&self.current)
		} else {
			None
		}
	}

	fn set_current(&mut self, state: Self::State) {
		if state.iter().all(|el| self.has_state(el)) {
			self.current = state;
		} else {
			self.current = HashSet::new();
		}
	}

	fn accepts(&self) -> bool {
		self.current
			.iter()
			.any(|el| self.get_state(el).unwrap().accepts)
	}

	fn step(&mut self, input: &I) {
		let mut new = HashSet::with_capacity(self.current.len());
		for el in &self.current {
			if let Some(states) = self.get_state(el).unwrap().transitions.get(input) {
				new = new.union(&states).cloned().collect();
			}
		}
		new.shrink_to_fit();
		self.current = new;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use maplit::hashset;

	#[test]
	fn construct() {
		// construct a simple DFA
		let mut nfa = NFA::<u32, char>::with_state(0, false);
		nfa.add_state(1, true);
		nfa.add_transition(0, 'a', 0).unwrap();
		nfa.add_transition(0, 'a', 1).unwrap();

		// check states
		assert!(nfa.has_state(&0), "Initially added state missing");
		assert!(nfa.has_state(&1), "Later added state missing");
		assert!(!nfa.accepts(), "Initial state incorrectly accepting");
		assert_eq!(
			Some(&hashset![0]),
			nfa.get_current(),
			"Initial state not set correctly"
		);
	}

	#[test]
	fn run() {
		let mut nfa = NFA::<u8, char>::with_state(0, false);
		nfa.add_state(1, false);
		nfa.add_state(2, true);
		nfa.add_transition(0, 'a', 1).unwrap();
		nfa.add_transition(0, 'a', 2).unwrap();
		nfa.add_transition(1, 'b', 1).unwrap();

		nfa.set_current(hashset![0, 1]);
		assert_eq!(
			Some(&hashset![0, 1]),
			nfa.get_current(),
			"Incorrect state after valid state set"
		);

		nfa.set_current(hashset![2, 4]);
		assert_eq!(
			None,
			nfa.get_current(),
			"Incorrect state after invalid state set"
		);

		nfa.set_current(hashset![0]);
		assert!(
			nfa.run(&"a".chars().collect::<Vec<_>>()),
			"Incorrect result on accepting run"
		);
		assert_eq!(
			Some(&hashset![0]),
			nfa.get_current(),
			"Incorrect state after run"
		);
	}

	#[test]
	fn deserialize() {
		let yaml = r"{states: {0: {accepts: false, transitions: {a: [0, 1], b: [1]}}, 1: {accepts: true}}, current: [0]}";
		let mut nfa: NFA<u8, char> = serde_yaml::from_str(yaml).unwrap();
		assert!(nfa.has_state(&0), "Deserialized DFA is missing state 0");
		assert!(
			nfa.run(&"aaa".chars().collect::<Vec<_>>()),
			"Incorrect result after run"
		);
	}
}
