#[cfg(feature = "quickcheck")]
fn main() {
    use core::{iter, time::Duration};
    use inator::*;
    use quickcheck::*;
    use std::{
        env,
        sync::{mpsc, Arc, Mutex},
        thread,
    };

    macro_rules! time {
        ($ex:expr) => {{
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || tx.send($ex).expect("Couldn't send to the main thread"));
            thread::sleep(Duration::from_millis(100));
            rx.try_recv().expect("Time's up!")
        }};
    }

    let gen_size = env::var("QUICKCHECK_GENERATOR_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    let qc_tests = 100000;
    // env::var("QUICKCHECK_TESTS")
    //     .ok()
    //     .and_then(|s| s.parse().ok())
    //     .unwrap_or(100);

    println!("Running {qc_tests} tests");

    let g = Arc::new(Mutex::new(Gen::new(gen_size)));
    for i in 0..qc_tests {
        println!("{i}");
        let gc = Arc::clone(&g);
        let both = Arc::new(time!(Vec::arbitrary(&mut gc.lock().unwrap())));
        let gc = Arc::clone(&g);
        let parser = Arc::new(time!(Nondeterministic::<u8, u8>::arbitrary(
            &mut gc.lock().unwrap()
        )));
        let pc = Arc::clone(&parser);
        time!(pc.check()).unwrap();
        let pc = Arc::clone(&parser);
        if time!(pc.accept(iter::empty()).is_err()) {
            continue;
        }
        let pc = Arc::clone(&parser);
        let bc = Arc::clone(&both);
        let sliceable = time!(sliceable(&pc, &bc));
        let first_half = time!(fixpoint("da capo") >> Arc::into_inner(parser).unwrap());
        let repeated = Arc::new(time!(first_half >> recurse("da capo")));
        let rc = Arc::clone(&repeated);
        time!(rc.check()).unwrap();
        let output = time!(repeated.accept(Arc::into_inner(both).unwrap()));
        if matches!(output, Err(ParseError::BadParser(_))) {
            continue;
        }
        assert_eq!(output.is_ok(), sliceable, "{output:?}");
    }
}

#[cfg(not(feature = "quickcheck"))]
fn main() {
    println!("Feature `quickcheck` disabled; doing nothing.")
}
