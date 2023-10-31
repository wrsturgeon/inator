#[cfg(feature = "quickcheck")]
fn main() {
    use core::{iter, time::Duration};
    use inator::*;
    use quickcheck::*;
    use std::{
        env, panic,
        sync::{mpsc, Arc},
        thread,
    };

    macro_rules! time {
        ($ex:expr) => {{
            let (tx_o, rx_o) = mpsc::channel();
            let (tx_t, rx_t) = mpsc::channel();
            let _rslt = thread::spawn(move || {
                let out = $ex;
                tx_t.send(())
                    .expect("Couldn't send to the stopwatch thread");
                tx_o.send(out).expect("Couldn't send to the main thread");
            });
            let _time = thread::spawn(move || {
                thread::sleep(Duration::from_millis(1000));
                rx_t.try_recv().expect("Time's up!");
            });
            rx_o.recv()
                .expect("Couldn't receive a value from the result thread")
        }};
    }

    let gen_size = env::var("QUICKCHECK_GENERATOR_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    // let qc_tests = env::var("QUICKCHECK_TESTS")
    //     .ok()
    //     .and_then(|s| s.parse().ok())
    //     .unwrap_or(100);

    fn check(both: &[u8], parser: &Nondeterministic<u8, u8>) {
        parser.check().unwrap();
        if parser.accept(iter::empty()).is_err() {
            return;
        }
        let sliceable = sliceable(&parser, &both);
        let first_half = fixpoint("da capo") >> parser.clone();
        let repeated = Arc::new(time!(first_half >> recurse("da capo"))); // <-- This is the infinite loop!
        repeated.check().unwrap();
        let output = repeated.accept(both.iter().copied());
        if matches!(output, Err(ParseError::BadParser(_))) {
            return;
        }
        assert_eq!(output.is_ok(), sliceable, "{output:?}");
    }

    let mut gen = Gen::new(gen_size);
    // for _ in 0..qc_tests {
    loop {
        let both = Vec::arbitrary(&mut gen);
        let parser = Nondeterministic::<u8, u8>::arbitrary(&mut gen);
        if panic::catch_unwind(|| check(&both, &parser)).is_err() {
            panic!(
                "
Parser:
{parser:?}

Input:
{both:?}
",
            );
        }
    }
}

#[cfg(not(feature = "quickcheck"))]
fn main() {
    println!("Feature `quickcheck` disabled; doing nothing.")
}
