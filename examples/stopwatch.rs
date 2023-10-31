#[cfg(feature = "quickcheck")]
fn main() {
    use core::{iter, time::Duration};
    use inator::*;
    use quickcheck::*;
    use std::{
        env, panic,
        sync::{mpsc, Arc, Mutex},
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
                thread::sleep(Duration::from_millis(100));
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

    /*
    let qc_tests = env::var("QUICKCHECK_TESTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    */

    fn check(both: Arc<Vec<u8>>, parser: Arc<Nondeterministic<u8, u8>>) {
        let pc = Arc::clone(&parser);
        time!(pc.check()).unwrap();
        let pc = Arc::clone(&parser);
        if time!(pc.accept(iter::empty()).is_err()) {
            return;
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
            return;
        }
        assert_eq!(output.is_ok(), sliceable, "{output:?}");
    }

    let g = Arc::new(Mutex::new(Gen::new(gen_size)));
    // for i in 0..qc_tests {
    loop {
        let gc = Arc::clone(&g);
        let both = Arc::new(time!(Vec::arbitrary(&mut gc.lock().unwrap())));
        let gc = Arc::clone(&g);
        let parser = Arc::new(time!(Nondeterministic::<u8, u8>::arbitrary(
            &mut gc.lock().unwrap()
        )));
        panic::catch_unwind(|| check(both, parser)).expect(
            "
Parser:
{parser:?}

Input:
{both:?}
",
        );
    }
}

#[cfg(not(feature = "quickcheck"))]
fn main() {
    println!("Feature `quickcheck` disabled; doing nothing.")
}
