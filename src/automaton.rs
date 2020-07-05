use std::fmt;

/// Trait representing an abstract automaton.
pub trait Automaton<S, I>
where
	Self: Default,
	S: Clone + PartialEq + fmt::Debug,
{
	/// Internal state type.
	type State: Clone;

	/// Creates a new empty automaton.
	fn new() -> Self {
		Self::default()
	}

	/// Creates a new automaton with a given initial state.
	fn with_state(id: S, accept: bool) -> Self {
		let mut automaton = Self::new();
		automaton.add_state(id.clone(), accept);
		automaton.set_current(Self::new_state(id));
		automaton
	}

	/// Creates a new state.
	fn new_state(id: S) -> Self::State;

	/// Creates a new automaton with a given set of states.
	fn from_states<V>(initial: Self::State, states: V) -> Self
	where
		V: IntoIterator<Item = (S, bool)>,
	{
		let mut automaton = Self::new();
		for (id, accept) in states {
			automaton.add_state(id, accept);
		}
		automaton.set_current(initial);
		automaton
	}

	/// Creates a new automaton with a given set of states & transitions.
	fn from_transitions<V, T>(
		initial: Self::State,
		states: V,
		transitions: T,
	) -> Result<Self, AutomatonError<S>>
	where
		V: IntoIterator<Item = (S, bool)>,
		T: IntoIterator<Item = (S, I, S)>,
	{
		let mut automaton = Self::from_states(initial, states);
		for (prev, input, next) in transitions {
			automaton.add_transition(prev, input, next)?;
		}
		Ok(automaton)
	}

	/// Checks whether the states of the automaton includes a state.
	fn has_state(&self, id: &S) -> bool;

	/// Adds a new state to the automaton.
	fn add_state(&mut self, id: S, accept: bool);

	/// Adds a new transition to the automaton.
	/// Returns an `AutomatonError::InexistentState` error if one of the states is inexistent.
	fn add_transition(&mut self, prev: S, input: I, next: S) -> Result<(), AutomatonError<S>>;

	/// Updates the current state.
	/// If the automaton does not have the passed state, it will go into an invalid state.
	fn set_current(&mut self, state: Self::State);

	/// Gets the current state.
	/// Returns None if the current state is invalid.
	fn get_current(&self) -> Option<&Self::State>;

	/// Checks whether the current state is accepting.
	fn accepts(&self) -> bool;

	/// Performs a single state transition.
	fn step(&mut self, input: &I);

	/// Runs the automaton on a sequence of inputs.
	/// This automatically resets the automaton after the execution.
	fn run<'a, V>(&mut self, inputs: V) -> bool
	where
		V: IntoIterator<Item = &'a I>,
		I: 'a,
	{
		match self.get_current() {
			Some(state) => {
				let state = state.clone();
				for input in inputs {
					self.step(input);
				}
				let result = self.accepts();
				self.set_current(state);
				result
			}
			None => false,
		}
	}
}

/// Enum representing an error.
#[derive(Debug)]
pub enum AutomatonError<S>
where
	S: fmt::Debug,
{
	InexistentState(S),
}

impl<S> fmt::Display for AutomatonError<S>
where
	S: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::InexistentState(state) => write!(f, "Inexistent State ID \"{:?}\"", state),
		}
	}
}
