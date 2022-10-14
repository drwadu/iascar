#![deny(clippy::all)]

mod counting;
mod transpiler;
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
        Some("-fa") => {
            unsafe { read_to_string(flags.next().unwrap_or(&"".to_owned())).unwrap_unchecked() }
                .lines()
                .map(|l| i32::from_str(l).ok())
                .flatten()
                .collect::<Vec<_>>()
        }
        _ => vec![],
    };
    /*
    let assumptions = match matches!(flags.next().map(String::as_str), Some("-a"))
        || matches!(flag.map(String::as_str), Some("-a"))
    {
        true => {
            let first = rest.next();

            match first.map(|input| i32::from_str(input)) {
                Some(Ok(lit)) => {
                    let mut delta = vec![Ok(lit)];

                    for lit in rest {
                        delta.push(i32::from_str(lit));
                    }

                    //#[cfg(feature = "verbose")]
                    //{
                    //    let v = delta.into_iter().flatten().collect::<Vec<i32>>();
                    //    print!("c o");
                    //    v.iter().for_each(|a| print!(" {:?}", a));
                    //    println!();
                    //    v
                    //}
                    //#[cfg(not(feature = "verbose"))]
                    delta.into_iter().flatten().collect::<Vec<i32>>()
                }
                //_ => first
                //    .and_then(|path| {
                //        read_to_string(path)
                //            .map(|delta| {
                //                delta
                //                    .lines()
                //                    .next()
                //                    .unwrap_or("")
                //                    .split_whitespace()
                //                    .flat_map(|lit| i32::from_str(lit).ok())
                //                    .collect::<Vec<_>>()
                //            })
                //            .ok()
                //    })
                //    .unwrap_or_default(),
                _ => vec![],
            }
        }
        _ => vec![],
    };
    */

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
                    println!("c s log10-estimate arb int {:?}", count.to_f32().log10());
                    println!("c s exact arb int {:?}", count);
                } else {
                    println!("s UNSATISFIABLE")
                }
            }
        }
        Some("-cnnfasp") => {
            println!(
                "{:?}",
                counting::count_on_sddnnf_asp(nnf_file, &assumptions)
            );
        }
        Some("-as") => {
            let mut dpcs_file = nnf_file.clone();
            dpcs_file = format!(
                "{}.cycles",
                dpcs_file.split('.').next().expect("no cycles file found.")
            );
            let dpcs = read_to_string(dpcs_file).expect("error occurred during reading cycles.");
            let mut lines = dpcs.lines();
            let no_bounding = lines
                .next()
                .and_then(|l| l.split_whitespace().next())
                .map(|n| usize::from_str(n).ok())
                .expect("invalid cycles file")
                == Some(0);
            let depth = rest_
                .next()
                .map_or(Some(0), |n| usize::from_str(n).ok())
                .expect("error occurred during reading alternation depth.");
            if !no_bounding {
                println!(
                    "{:?}",
                    counting::count_on_cg_with_cycles(nnf_file, lines, &assumptions, depth)
                );
            } else {
                println!("{:?}", counting::count_on_cg(nnf_file, &assumptions));
            }
        }
        Some("-t") => {
            let transpilation = transpiler::transpile(nnf_file);

            if let Err(e) = transpilation {
                println!("transpiling failed: {:?}", e);
                std::process::exit(-1)
            }
        }
        _ => {
            #[cfg(not(feature = "verbose"))]
            println!("{:?}", counting::count_on_cg(nnf_file, &assumptions));

            #[cfg(feature = "verbose")]
            {
                let count = counting::count_on_cg(nnf_file, &assumptions);
                if count > rug::Integer::from(0) {
                    println!("s SATISFIABLE");
                    println!("c s type cccg");
                    println!("c s log10-estimate {:?}", count.to_f64().log10());
                    println!("c s exact arb int {:?}", count);
                } else {
                    println!("s UNSATISFIABLE")
                }
            }
        }
    }
}
