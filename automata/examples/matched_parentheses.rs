use inator_automata::{dyck_d, Run};
use rand::{thread_rng, RngCore};

/// Generate test cases (has nothing to do with automata!).
fn generate<R: RngCore>(rng: &mut R, fuel: u8) -> String {
    let Some(depleted) = fuel.checked_sub(1) else {
        return String::new();
    };
    let f: [fn(&mut R, u8) -> String; 3] = [
        |_, _| String::new(),
        |r, d| "(".to_owned() + &generate(r, d) + ")",
        |r, d| generate(r, d >> 1) + &generate(r, d >> 1),
    ];
    f[(rng.next_u32() % 3) as usize](rng, depleted)
}

/// Check if this string consists of matched parentheses.
fn accept<I: Iterator<Item = char>>(iter: I) -> bool {
    let mut i: usize = 0;
    for c in iter {
        i = match c {
            '(' => i + 1,
            ')' => {
                if let Some(pred) = i.checked_sub(1) {
                    pred
                } else {
                    return false;
                }
            }
            _ => unreachable!(),
        }
    }
    i == 0
}

/// Output a jumble of parentheses with a very low chance of being valid.
fn shitpost<R: RngCore>(rng: &mut R) -> String {
    let mut s = String::new();
    loop {
        let i = rng.next_u32();
        if (i & 2) == 0 {
            return s;
        }
        s.push(if (i & 1) == 0 { '(' } else { ')' });
    }
}

pub fn main() {
    let parser = dyck_d();

    let mut rng = thread_rng();

    // Accept all valid strings
    for _ in 0..10 {
        let s = generate(&mut rng, 32);
        println!();
        println!("\"{s}\"");
        let mut run = s.chars().run(&parser);
        println!("    {run:?}");
        while let Some(r) = run.next() {
            let Ok(c) = r else { panic!("{r:?}") };
            println!("{c:?} {run:?}");
        }
    }

    // Reject all invalid strings
    'examples: for _ in 0..10 {
        let s = shitpost(&mut rng);
        println!();
        println!("\"{s}\"");
        let mut run = s.chars().run(&parser);
        println!("    {run:?}");
        while let Some(r) = run.next() {
            let Ok(c) = r else {
                assert!(!accept(s.chars()));
                continue 'examples;
            };
            println!("{c:?} {run:?}");
        }
        assert!(accept(s.chars()));
    }

    // Print the Rust source representation of this parser
    println!("{}", parser.to_src().unwrap());
}
