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

    let qc_tests = env::var("QUICKCHECK_TESTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    fn check(parser: &Nondeterministic<u8, u8>) -> bool {
        let first_half = fixpoint("da capo") >> parser.clone();
        time!(first_half >> recurse("da capo")).is_some()
    }

    let mut gen = Gen::new(gen_size);
    for _ in 0..qc_tests {
        let parser = Nondeterministic::arbitrary(&mut gen);
        if !check(&parser) {
            for shrunk in parser.shrink() {
                if !check(&shrunk) {
                    panic!(
                        "
Parser:
{:?}

",
                        shrunk.to_src(),
                    )
                }
            }
            panic!(
                "
Parser:
{:?}

",
                parser.to_src(),
            )
        }
    }
}

#[cfg(not(feature = "quickcheck"))]
fn main() {
    use core::marker::PhantomData;
    use inator::*;
    use std::collections::{BTreeMap, BTreeSet};

    let parser = Nondeterministic {
        states: vec![
            State {
                transitions: CurryStack {
                    wildcard: Some(CurryInput::Wildcard(Transition {
                        dst: core::iter::once(Ok(0)).collect(),
                        act: Action::Local,
                        update: Update {
                            input_t: "()".to_owned(),
                            output_t: "()".to_owned(),
                            ghost: PhantomData::<u8>,
                            src: "|(), _| {}",
                        },
                    })),
                    map_none: None,
                    map_some: BTreeMap::new(),
                },
                non_accepting: BTreeSet::new(),
                tags: ["$6f".to_owned(), "'x".to_owned(), "2-".to_owned()]
                    .into_iter()
                    .collect(),
            },
            State {
                transitions: CurryStack {
                    wildcard: Some(CurryInput::Wildcard(Transition {
                        dst: [Ok(0), Ok(2)].into_iter().collect(),
                        act: Action::Push(b'o'),
                        update: Update {
                            input_t: "()".to_owned(),
                            output_t: "()".to_owned(),
                            ghost: PhantomData,
                            src: "|(), _| {}",
                        },
                    })),
                    map_none: None,
                    map_some: vec![
                        (
                            b'\xa5',
                            CurryInput::Scrutinize(RangeMap {
                                entries: BTreeMap::new(),
                            }),
                        ),
                        (
                            b'\xd6',
                            CurryInput::Scrutinize(RangeMap {
                                entries: BTreeMap::new(),
                            }),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                },
                non_accepting: [";8".to_owned(), "ff".to_owned(), "6k".to_owned()]
                    .into_iter()
                    .collect(),
                tags: ["cG2".to_owned(), "fb".to_owned()].into_iter().collect(),
            },
            State {
                transitions: CurryStack {
                    wildcard: Some(CurryInput::Wildcard(Transition {
                        dst: core::iter::once(Ok(1)).collect(),
                        act: Action::Push(b'\x81'),
                        update: Update {
                            input_t: "()".to_owned(),
                            output_t: "()".to_owned(),
                            ghost: PhantomData,
                            src: "|(), _| {}",
                        },
                    })),
                    map_none: None,
                    map_some: BTreeMap::new(),
                },
                non_accepting: core::iter::once("".to_owned()).collect(),
                tags: BTreeSet::new(),
            },
        ],
        initial: core::iter::once(Ok(0)).collect(),
    };

    let _ = fixpoint("da capo") >> parser.clone() >> recurse("da capo");
}
