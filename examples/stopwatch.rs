#[cfg(feature = "quickcheck")]
fn main() {
    use core::{iter, time::Duration};
    use inator::*;
    use quickcheck::*;
    use std::{env, panic, sync::mpsc, thread, time::Instant};

    macro_rules! time {
        ($ex:expr) => {
            (|| {
                let (tx, rx) = mpsc::channel();
                let now = Instant::now();
                thread::Builder::new()
                    .name("Worker".to_owned())
                    .spawn(move || {
                        let out = $ex;
                        tx.send(out).expect("Couldn't send to the main thread");
                    })
                    .expect("Couldn't start another thread");
                while now.elapsed() < Duration::from_millis(1000) {
                    if let Ok(ok) = rx.try_recv() {
                        return Some(ok);
                    }
                }
                None
            })()
        };
    }

    let gen_size = env::var("QUICKCHECK_GENERATOR_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    // let qc_tests = env::var("QUICKCHECK_TESTS")
    //     .ok()
    //     .and_then(|s| s.parse().ok())
    //     .unwrap_or(100);

    fn check(both: Vec<u8>, parser: Nondeterministic<u8, u8>) {
        parser.check().unwrap();
        if parser.accept(iter::empty()).is_err() {
            return;
        }
        let sliceable = sliceable(&parser, &both);
        let first_half = fixpoint("da capo") >> parser.clone();
        let Some(repeated) = time!(first_half >> recurse("da capo")) else {
            panic!(
                "
Parser:
{parser:?}

Input:
{both:?}
",
            )
        };
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
        check(both, parser);
    }
}

#[cfg(not(feature = "quickcheck"))]
fn main() {
    println!("Feature `quickcheck` disabled; doing nothing.")
}
