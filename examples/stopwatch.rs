#[cfg(feature = "quickcheck")]
fn main() {
    use core::time::Duration;
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
                while now.elapsed() < Duration::from_secs(10) {
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

    fn check(parser: &Nondeterministic<u8, u8>) -> bool {
        let first_half = fixpoint("da capo") >> parser.clone();
        time!(first_half >> recurse("da capo")).is_some()
    }

    let mut gen = Gen::new(gen_size);
    // for _ in 0..qc_tests {
    loop {
        let parser = Nondeterministic::arbitrary(&mut gen);
        if !check(&parser) {
            for shrunk in parser.shrink() {
                if !check(&shrunk) {
                    panic!(
                        "
Parser:
{shrunk:?}

",
                    )
                }
            }
            panic!(
                "
Parser:
{parser:?}

",
            )
        }
    }
}

#[cfg(not(feature = "quickcheck"))]
fn main() {
    println!("Feature `quickcheck` disabled; doing nothing.")
}
