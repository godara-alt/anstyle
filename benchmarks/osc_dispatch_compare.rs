// Temporary benchmark harness for comparing original vs safe OSC dispatch.
// Identical source is compiled against both revisions in the same CI runner.
use anstyle_parse::{DefaultCharAccumulator, Parser, Perform};
use std::hint::black_box;
use std::time::Instant;

#[derive(Default)]
struct Observer {
    digest: u64,
}

impl Perform for Observer {
    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        let total_bytes: usize = params.iter().map(|p| p.len()).sum();
        self.digest = self
            .digest
            .wrapping_add(total_bytes as u64)
            .wrapping_add(params.len() as u64)
            .wrapping_add(bell_terminated as u64);
    }
}

fn run(input: &[u8], iterations: usize) -> u64 {
    let mut parser = Parser::<DefaultCharAccumulator>::new();
    let mut observer = Observer::default();
    let start = Instant::now();
    for _ in 0..iterations {
        for &byte in input {
            parser.advance(&mut observer, byte);
        }
    }
    black_box(observer.digest);
    start.elapsed().as_nanos() as u64
}

fn main() {
    // An empty OSC, a short common OSC, mixed empty fields, and the 16-field limit.
    let cases: &[(&str, &[u8])] = &[
        ("empty", b"\x1b]\x07"),
        ("short", b"\x1b]2;hello\x07"),
        ("mixed_empty", b"\x1b]2;first;;last\x07"),
        ("max_16", b"\x1b]0;1;2;3;4;5;6;7;8;9;10;11;12;13;14;15\x07"),
    ];
    println!("case,median_ns_per_osc,min_ns_per_osc,max_ns_per_osc,iterations,samples");
    for &(name, raw) in cases {
        // Prevent compile-time specialization of the input.
        let input = black_box(raw);
        run(input, 25_000);
        const N: usize = 7;
        const ITERATIONS: usize = 500_000;
        let mut samples = [0f64; N];
        for sample in &mut samples {
            let elapsed_ns = run(input, ITERATIONS);
            *sample = elapsed_ns as f64 / ITERATIONS as f64;
        }
        samples.sort_by(|a, b| a.total_cmp(b));
        println!(
            "{},{:.3},{:.3},{:.3},{},{}",
            name, samples[N / 2], samples[0], samples[N - 1], ITERATIONS, N
        );
    }
}
