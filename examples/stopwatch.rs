fn main() {
    #[cfg(not(feature = "quickcheck"))]
    {
        println!("`quickcheck` feature not enabled; passing...");
    }
    #[cfg(feature = "quickcheck")]
    {
        use quickcheck::{Arbitrary, Gen};
        use std::{
            env,
            io::{stdout, Write},
            time::Instant,
        };
        use vpa::Nondeterministic;

        let mut g = Gen::new(
            env::var("QUICKCHECK_GENERATOR_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
        );
        for _ in 0..env::var("QUICKCHECK_TESTS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100)
        {
            print!("Generating...");
            stdout().flush().expect("Couldn't flush stdout");
            let mut clock = Instant::now();
            let nd = Nondeterministic::<bool, bool>::arbitrary(&mut g);
            println!("done in {:?}", clock.elapsed());
            print!("Determinizing...");
            stdout().flush().expect("Couldn't flush stdout");
            clock = Instant::now();
            let _ = nd.determinize();
            println!("done in {:?}", clock.elapsed());
        }
    }
}
