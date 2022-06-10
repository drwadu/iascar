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

    let mut inputs = args.collect::<Vec<_>>();
    let split_idx = partition(&mut inputs, |directive| {
        matches!(
            directive.chars().nth(1).map(|c| c.is_alphabetic()),
            Some(true)
        )
    });

    let mut flags = inputs[..split_idx].iter();
    let mut rest = inputs[split_idx..].iter();
    let flag = flags.next();

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

                    delta.into_iter().flatten().collect::<Vec<i32>>()
                }
                _ => first
                    .and_then(|path| {
                        read_to_string(path)
                            .map(|delta| {
                                delta
                                    .lines()
                                    .next()
                                    .unwrap_or("")
                                    .split_whitespace()
                                    .flat_map(|lit| i32::from_str(lit).ok())
                                    .collect::<Vec<_>>()
                            })
                            .ok()
                    })
                    .unwrap_or_default(),
            }
        }
        _ => vec![],
    };

    match flag.map(String::as_str) {
        Some("-cnnf") => {
            println!("{:?}", counting::count_on_sddnnf(nnf_file, &assumptions));
        }
        Some("-cnnfasp") => {
            println!("{:?}", counting::count_on_sddnnf_asp(nnf_file, &assumptions));
        }
        Some("-as") => {
            let mut dpcs_file = nnf_file.clone();
            dpcs_file = format!(
                "{}.cycles",
                dpcs_file
                    .split('.')
                    .next()
                    .expect("no .dpcs found.")
            );
            let dpcs = read_to_string(dpcs_file).expect("unkown error.");
            let mut lines = dpcs.lines();
            let no_bounding = lines
                .next()
                .and_then(|l| l.split_whitespace().next())
                .map(|n| usize::from_str(n).ok())
                .unwrap() // TODO
                == Some(0);
            if !no_bounding {
                println!(
                    "{:?}",
                    counting::count_on_cg_with_cycles(nnf_file, lines, &assumptions, 0)
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
            println!("{:?}", counting::count_on_cg(nnf_file, &assumptions));
        }
    }
}
