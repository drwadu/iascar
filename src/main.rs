#![deny(clippy::all)]

mod counting;
mod utils;

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

    let inputs = args.collect::<Vec<_>>();
    let mut iter = inputs.iter();
    let assumptions = match iter.next().map(String::as_str) {
        Some("--a") => inputs[1..]
            .iter()
            .map(String::as_str)
            .map(i32::from_str)
            .flatten()
            .collect::<Vec<_>>(),
        Some("--fa") => {
            read_to_string(iter.next().unwrap_or(&"".to_owned())) // unwrap_unchecked
                .unwrap()
                .lines()
                .map(|l| i32::from_str(l).ok())
                .flatten()
                .collect::<Vec<_>>()
        }
        _ => vec![],
    };

    if inputs.contains(&"--cccg".to_owned()) {
        #[cfg(not(feature = "verbose"))]
        println!("{:?}", counting::count_on_ccg_io(nnf_file, &assumptions));
        #[cfg(feature = "verbose")]
        {
            let count = counting::count_on_ccg_io(nnf_file, &assumptions);
            if count > rug::Integer::from(0) {
                println!("s SATISFIABLE");
                println!("c s log10-estimate {:?}", count.to_f64().log10());
                println!("c s exact arb int {:?}", count);
            } else {
                println!("s UNSATISFIABLE")
            }
        }
    } else if inputs.contains(&"--cnnf".to_owned()) {
        #[cfg(not(feature = "verbose"))]
        println!("{:?}", counting::count_on_sddnnf(nnf_file, &assumptions));
        #[cfg(feature = "verbose")]
        {
            let count = counting::count_on_sddnnf(nnf_file, &assumptions);
            if count > rug::Integer::from(0) {
                println!("s SATISFIABLE");
                println!("c s log10-estimate {:?}", count.to_f64().log10());
                println!("c s exact arb int {:?}", count);
            } else {
                println!("s UNSATISFIABLE")
            }
        }
    } else if inputs.contains(&"--cnnfasp".to_owned()) {
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
    } else {
        let mut dpcs_file = nnf_file.clone();
        dpcs_file = format!("{}.ucs", dpcs_file);
        let dpcs = read_to_string(dpcs_file).expect("error occurred during reading cycles.");
        let lines = dpcs.lines();
        let depth = match inputs.iter().find(|s| s.starts_with("--d=")) {
            Some(s) => s.split('=').next().map(usize::from_str).unwrap().unwrap(),
            _ => 0,
        };

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
