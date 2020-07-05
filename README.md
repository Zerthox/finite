# Finite
A small library for finite state automatons.

## Features
- Generic type parameters for automaton state & input.
- Extensible `Automaton` trait providing the main functionality.
- Conversion between `DFA` and `NFA`.
- Integration with [serde](https://serde.rs/) for easy conversion from & to data formats.

## Usage
```rust
// bring finite into scope
use finite::{Automaton, DFA};

// create a new DFA with initial non-accepting state 0
let mut dfa = DFA::with_state(0, false);

// add a new accepting state 1
dfa.add_state(1, true);

// add two transitions
let transition = (0, 'a', 1);
dfa.add_transition(transition);
dfa.add_transition((1, 'b', 1));

// run the dfa on a sequence of inputs
let input = ['a', 'b', 'b', 'b'];
let result = dfa.run(&input);
```

Automatons have generic type parameters for state & input, so you can use custom structs in DFA & NFAs as long as they implement a handful of required traits. In this example we also use the [maplit](https://docs.rs/maplit/) crate to easily construct a DFA from a nested hash map.

```rust
use finite::{Automaton, DFA};
use maplit::hashmap;

// custom struct with required traits
#[derive(Default, Clone, PartialEq, Eq, Hash, Debug)]
struct Custom(i32, u8);

// create a new DFA from a map
let initial = Custom(12, 5);
let mut dfa: DFA<Custom, &str> = DFA::from_map(initial, hashmap!(
	Custom(12, 5) => (
		true,
		hashmap!(
			"abc" => Custom(-24, 6)
		)
	),
	Custom(-24, 6) => (
		false,
		hashmap!(
			"bar" => Custom(-24, 6),
			"foo" => Custom(12, 5)
		)
	)
));
let result = dfa.run(&["abc", "invalid"]);
```
