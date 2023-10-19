#![deny(clippy::all)]

mod compressor;
mod counter;
mod counting;
mod utils;

use std::env::Args;
use std::fs::read_to_string;
use std::iter::Skip;
use std::str::FromStr;

pub(crate) const AND: u8 = 1;
pub(crate) const OR: u8 = 0;
pub(crate) const SAND: &'static str = "*";
pub(crate) const SOR: &'static str = "+";

fn read_assumptions(mut args: Skip<Args>) -> Vec<i32> {
    match args.next().as_deref() {
        Some("-a") => args
            .map(|l| i32::from_str(l.trim()).ok())
            .flatten()
            .collect::<Vec<_>>(),
        Some("-fa") => args
            .next()
            .and_then(|f| read_to_string(f).ok())
            .map(|s| {
                s.split_whitespace()
                    .map(|l| i32::from_str(l.trim()).ok())
                    .flatten()
                    .collect::<Vec<_>>()
            })
            .unwrap_or(vec![]),
        _ => vec![],
    }
}

fn main() {
    let mut args = std::env::args().skip(1);

    #[allow(unreachable_code)]
    match args.next().as_deref() {
        Some("-ccg") => args
            .next()
            .and_then(|s| if s.trim() == "-in" { args.next() } else { None })
            .map_or_else(
                || {
                    println!("error: provide ccg file path with {:?}.", "-in path");
                    std::process::exit(-1)
                },
                |f| {
                    let count = counting::count_on_ccg_io(f, &read_assumptions(args));
                    if count > rug::Integer::from(0) {
                        println!("s SATISFIABLE");
                        println!("c s log10-estimate {:?}", count.to_f64().log10());
                        println!("c s exact arb int {:?}", count);
                    } else {
                        println!("s UNSATISFIABLE")
                    }
                },
            ),
        Some("-com") => args
            .next()
            .and_then(|s| if s == "-lp" { args.next() } else { None })
            .zip({
                if args.next().as_deref() == Some("-cnf") {
                    args.next()
                } else {
                    None
                }
            })
            .zip({
                if args.next().as_deref() == Some("-nnf") {
                    args.next()
                } else {
                    None
                }
            })
            .map(|((lp, cnf), nnf)| compressor::compress_(nnf, lp, cnf))
            .unwrap_or_else(|| {
                println!(
                    "error: please provide input in the following order {:?}.",
                    "-lp logic_program_path -cnf cnf_path -nnf nnf_path"
                );
                std::process::exit(-1)
            })
            .err()
            .map(|err| {
                println!("error: {:?}.", err.to_string());
                std::process::exit(-1)
            })
            .unwrap_or(()),
        Some("-car") => args
            .next()
            .and_then(|s| if s == "-ccg" { args.next() } else { None })
            .zip({
                if args.next().as_deref() == Some("-ucs") {
                    args.next().and_then(|f| read_to_string(f).ok())
                } else {
                    None
                }
            })
            .zip({
                if args.next().as_deref() == Some("-dep") {
                    match args.next().as_deref().map(usize::from_str) {
                        Some(Ok(u)) => Some(u),
                        Some(Err(e)) => {
                            println!("error: {:?}.", e.to_string());
                            std::process::exit(-1)
                        }
                        _ => {
                            println!("error: provide depth with {:?}.", "-dep int");
                            std::process::exit(-1)
                        }
                    }
                } else {
                    Some(0)
                }
            })
            .map_or_else(
                || {
                    println!(
                        "error: please provide input in the following order {:?}.",
                        "-ccg counting_graph -ucs unsupported_constraints -dep alternation_depth"
                    );
                    std::process::exit(-1)
                },
                |((ccg, ucs), dep)| {
                    let count =
                        counting::anytime_cg_count(ccg, ucs.lines(), &read_assumptions(args), dep);
                    if count > rug::Integer::from(0) {
                        println!("s SATISFIABLE");
                        println!("c s log10-estimate {:?}", count.to_f64().log10());
                        println!("c s exact arb int {:?}", count);
                    } else {
                        println!("s UNSATISFIABLE")
                    }
                },
            ),
        Some("-nnf") => args
            .next()
            .and_then(|s| if s.trim() == "-in" { args.next() } else { None })
            .map_or_else(
                || {
                    println!("error: provide nnf file path with {:?}.", "-in path");
                    std::process::exit(-1)
                },
                |f| {
                    let count = counting::count_on_sddnnf_asp(f, &read_assumptions(args));
                    if count > rug::Integer::from(0) {
                        println!("s SATISFIABLE");
                        println!("c s log10-estimate {:?}", count.to_f64().log10());
                        println!("c s exact arb int {:?}", count);
                    } else {
                        println!("s UNSATISFIABLE")
                    }
                },
            ),
        Some("-nnfarb") => args
            .next()
            .and_then(|s| if s.trim() == "-in" { args.next() } else { None })
            .map_or_else(
                || {
                    println!("error: provide nnf file path with {:?}.", "-in path");
                    std::process::exit(-1)
                },
                |f| {
                    let count = counting::count_on_sddnnf(f, &read_assumptions(args));
                    if count > rug::Integer::from(0) {
                        println!("s SATISFIABLE");
                        println!("c s log10-estimate {:?}", count.to_f64().log10());
                        println!("c s exact arb int {:?}", count);
                    } else {
                        println!("s UNSATISFIABLE")
                    }
                },
            ),
        Some(s) => {
            println!("error: unknown operation {:?}.", s);
            std::process::exit(-1)
        }
        _ => {
            println!("error: specify operation.");
            std::process::exit(-1)
        }
    }
}
