#![deny(clippy::all)]

mod counting;
mod utils;

use itertools::partition;

use std::fs::read_to_string;
use std::str::FromStr;

pub const AND: u8 = 1;
pub const OR: u8 = 0;

fn main() {
    let mut args = std::env::args().skip(1);

    let nnf_file = match args.next() {
        Some(path) => path,
        _ => {
            println!("\nprovide .nnf file path.\n");
            std::process::exit(-1)
        }
    };

    #[cfg(feature = "verbose")]
    {
        println!(
            "c o {} version {}\nc o reading from {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            nnf_file
        );
    }

    let mut inputs = args.collect::<Vec<_>>();
    let split_idx = partition(&mut inputs, |directive| {
        matches!(
            directive.chars().nth(1).map(|c| c.is_alphabetic()),
            Some(true)
        )
    });

    let mut flags = inputs[..split_idx].iter();
    let mut rest = inputs[split_idx..].iter();
    let mut rest_ = inputs[split_idx..].iter();
    let flag = flags.next();

    let assumptions = match flag.map(String::as_str) {
        Some("-a") => {
            let first = rest.next();

            match first.map(|input| i32::from_str(input)) {
                Some(Ok(lit)) => {
                    let mut delta = vec![Ok(lit)];

                    for lit in rest {
                        delta.push(i32::from_str(lit));
                    }

                    delta.into_iter().flatten().collect::<Vec<i32>>()
                }
                _ => vec![],
            }
        }
        Some("-fa") => read_to_string(flags.next().unwrap_or(&"".to_owned())) // unwrap_unchecked
            .unwrap()
            .lines()
            .map(|l| i32::from_str(l).ok())
            .flatten()
            .collect::<Vec<_>>(),
        _ => vec![],
    };

    match flag.map(String::as_str) {
        Some("-cnnf") => {
            #[cfg(not(feature = "verbose"))]
            println!("{:?}", counting::count_on_sddnnf(nnf_file, &assumptions));
            #[cfg(feature = "verbose")]
            {
                let count = counting::count_on_sddnnf(nnf_file, &assumptions);
                if count > rug::Integer::from(0) {
                    println!("s SATISFIABLE");
                    println!("c s type cnnf");
                    println!("c s log10-estimate {:?}", count.to_f64().log10());
                    println!("c s exact arb int {:?}", count);
                } else {
                    println!("s UNSATISFIABLE")
                }
            }
        }
        Some("-cnnfasp") => {
            #[cfg(not(feature = "verbose"))]
            println!(
                "{:?}",
                counting::count_on_sddnnf_asp(nnf_file, &assumptions)
            );
            #[cfg(feature = "verbose")]
            {
                let count = counting::count_on_sddnnf_asp(nnf_file, &assumptions);
                if count > rug::Integer::from(0) {
                    println!("s SATISFIABLE");
                    println!("c s type cnnfasp");
                    println!("c s log10-estimate {:?}", count.to_f64().log10());
                    println!("c s exact arb int {:?}", count);
                } else {
                    println!("s UNSATISFIABLE")
                }
            }
        }
        _ => {
            let mut dpcs_file = nnf_file.clone();
            dpcs_file = format!(
                "{}.ucs",
                dpcs_file.split('.').next().expect("no cycles file found.")
            );
            let dpcs = read_to_string(dpcs_file).expect("error occurred during reading cycles.");
            let lines = dpcs.lines();
            let depth = rest_
                .next()
                .map_or(Some(0), |n| usize::from_str(n).ok())
                .expect("error occurred during reading alternation depth.");

            #[cfg(not(feature = "verbose"))]
            println!(
                "{:?}",
                counting::anytime_cg_count(nnf_file, lines, &assumptions, depth)
            );

            #[cfg(feature = "verbose")]
            {
                let count = counting::anytime_cg_count(nnf_file, lines, &assumptions, depth);
                if count > rug::Integer::from(0) {
                    println!("s SATISFIABLE");
                    println!("c s log10-estimate {:?}", count.to_f64().log10());
                    println!("c s exact arb int {:?}", count);
                } else {
                    println!("s UNSATISFIABLE")
                }
            }
        }
    }
}
