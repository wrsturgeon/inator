use crate::*;

#[cfg(feature = "quickcheck")]
quickcheck::quickcheck! {
    fn nfa_dfa_equal(nfa: Nfa<u8>, input: Vec<Vec<u8>>) -> bool {
        println!("{nfa:?}");
        let dfa: Dfa<u8> = nfa.clone().into();
        input.into_iter().all(|v| nfa.accept(v.iter().cloned()) == dfa.accept(v))
    }
}
