#[cfg(any(debug_assertions, test))]
macro_rules! unwrap {
    ($expr:expr) => {
        $expr.unwrap()
    };
}

#[cfg(not(any(debug_assertions, test)))]
macro_rules! unwrap {
    ($expr:expr) => {
        unsafe { $expr.unwrap_unchecked() }
    };
}

mod dfa;
mod nfa;
mod powerset_construction;

#[cfg(test)]
mod test;

pub use {dfa::Graph as Dfa, nfa::Graph as Nfa};
