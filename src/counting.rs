use crate::utils::ToHashSet;
use itertools::Itertools;
//#[cfg(feature = "parallel")]
use rayon::prelude::*;
use rug::Integer;
use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::Path;
use std::str::FromStr;

pub fn count_on_sddnnf(filename: impl AsRef<Path>, assumptions: &[i32]) -> Integer {
    let nnf = read_to_string(&filename).unwrap_or_else(|_| "".to_string());

    let mut lines = nnf.lines();
    let mut stats = lines
        .next()
        .map(|line| line.split_whitespace())
        .expect("reading nnf stats failed");
    stats.next();
    let node_count = stats
        .next()
        .and_then(|s| usize::from_str(s).ok())
        .expect("reading node count failed.");
    stats.next();
    let var_count = stats
        .next()
        .and_then(|s| usize::from_str(s).ok())
        .expect("reading var count failed.");

    let (mut vals, mut vars) = (
        Vec::with_capacity(node_count),
        Vec::with_capacity(node_count),
    );

    let mut count = Integer::from(0);

    lines.for_each(|line| {
        let mut spec = line.split_whitespace();

        match spec.next() {
            Some("L") => {
                let lit = spec
                    .next()
                    .and_then(|l| i32::from_str(l).ok())
                    .expect("reading literal failed.");

                match assumptions.contains(&-lit) {
                    false => {
                        count = Integer::from(1);
                        vars.push(vec![Integer::from(lit).abs()].to_hashset());
                    }
                    _ => {
                        count = Integer::from(0);
                        vars.push(vec![].to_hashset());
                    }
                }

                vals.push(count.clone());
            }
            Some("A") => {
                let children_ids = spec
                    .skip(1)
                    .filter_map(|child| usize::from_str(child).ok())
                    .collect::<Vec<_>>();

                count = Integer::from(1);
                let mut node_vars = HashSet::new();
                children_ids.iter().for_each(|child_id| {
                    node_vars = node_vars
                        .union(unsafe { vars.get_unchecked(*child_id) })
                        .cloned()
                        .collect();
                    count *= unsafe { vals.get_unchecked(*child_id) };
                });

                vars.push(node_vars);
                vals.push(count.clone());
            }
            Some("O") => {
                count = Integer::from(0);

                let children_ids = spec
                    .skip(2)
                    .filter_map(|child| usize::from_str(child).ok())
                    .collect::<Vec<_>>();

                let node_vars = children_ids
                    .iter()
                    .map(|child_id| unsafe { vars.get_unchecked(*child_id) })
                    .fold(HashSet::new(), |acc, set| {
                        set.union(&acc).cloned().collect()
                    });
                let n_vars = node_vars.len();
                children_ids
                    .iter()
                    .map(|child_id| (child_id, unsafe { vals.get_unchecked(*child_id) }))
                    .for_each(|(child_id, val)| {
                        let gap_size = n_vars - unsafe { vars.get_unchecked(*child_id) }.len();
                        count += val.clone() << gap_size;
                    });

                vars.push(node_vars);
                vals.push(count.clone());
            }
            _ => (),
        }
    });

    let gap_size = var_count - unsafe { vars.get_unchecked(node_count - 1) }.len();
    count << gap_size
}

pub fn count_on_sddnnf_asp(filename: impl AsRef<Path>, assumptions: &[i32]) -> Integer {
    let nnf = read_to_string(&filename).unwrap_or_else(|_| "".to_string());

    let mut lines = nnf.lines();
    let mut stats = lines
        .next()
        .map(|line| line.split_whitespace())
        .expect("reading nnf stats failed");
    stats.next();
    let node_count = stats
        .next()
        .and_then(|s| usize::from_str(s).ok())
        .expect("reading node count failed.");

    let mut vals = Vec::with_capacity(node_count);

    let mut count = Integer::from(0);

    lines.for_each(|line| {
        let mut spec = line.split_whitespace();

        match spec.next() {
            Some("L") => {
                let lit = spec
                    .next()
                    .and_then(|l| i32::from_str(l).ok())
                    .expect("reading literal failed.");

                match assumptions.contains(&-lit) {
                    false => {
                        count = Integer::from(1);
                    }
                    _ => {
                        count = Integer::from(0);
                    }
                }

                vals.push(count.clone());
            }
            Some("A") => {
                let children_ids = spec
                    .skip(1)
                    .filter_map(|child| usize::from_str(child).ok())
                    .collect::<Vec<_>>();

                count = Integer::from(1);
                children_ids.iter().for_each(|child_id| {
                    count *= unsafe { vals.get_unchecked(*child_id) };
                });

                vals.push(count.clone());
            }
            Some("O") => {
                count = Integer::from(0);

                let children_ids = spec
                    .skip(2)
                    .filter_map(|child| usize::from_str(child).ok())
                    .collect::<Vec<_>>();

                children_ids
                    .iter()
                    .map(|child_id| unsafe { vals.get_unchecked(*child_id) })
                    .for_each(|val| {
                        count += val.clone();
                    });

                vals.push(count.clone());
            }
            _ => (),
        }
    });

    count
}

pub fn count_on_ccg_io(ccg: impl AsRef<Path>, assumptions: &[i32]) -> Integer {
    let ccg_nodes = read_to_string(ccg)
        .unwrap()
        .lines()
        .into_iter()
        .filter(|l| !l.starts_with('c'))
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let node_count = ccg_nodes.len();

    println!("c o a={:?}", assumptions);

    let mut nodes = Vec::with_capacity(node_count);

    let mut count = Integer::from(0);

    for node in ccg_nodes {
        let mut spec = node.split_whitespace();
        match spec.next() {
            Some("*") => {
                let n_children = spec
                    .next()
                    .and_then(|child| usize::from_str(child).ok())
                    .unwrap();

                let children_ = &spec
                    .flat_map(|child| usize::from_str(child).ok())
                    .collect::<Vec<_>>()[..n_children];
                let children = children_
                    .iter()
                    .map(|idx| unsafe { nodes.get_unchecked(*idx) })
                    .collect::<Vec<_>>();

                count = children
                    .iter()
                    .fold(Integer::from(1), |acc, child_val: &&Integer| {
                        acc * &(*child_val).clone()
                    });

                nodes.push(count.clone());
            }
            Some("+") => {
                count = Integer::from(0);
                let n_children = spec
                    .next()
                    .and_then(|child| usize::from_str(child).ok())
                    .unwrap();

                let children_ = &spec
                    .flat_map(|child| usize::from_str(child).ok())
                    .collect::<Vec<_>>()[..n_children];
                let children = children_
                    .iter()
                    .map(|idx| unsafe { nodes.get_unchecked(*idx) })
                    .collect::<Vec<_>>();

                children.iter().for_each(|child_val| {
                    count += &(*child_val).clone();
                });

                nodes.push(count.clone());
            }
            o => {
                let lit = o
                    .and_then(|l| i32::from_str(l).ok())
                    .expect("reading literal failed.");

                count = spec
                    .next()
                    .and_then(|l| i32::from_str(l).map(Integer::from).ok())
                    .expect("reading val failed.");

                match assumptions.contains(&-lit) {
                    false => {
                        nodes.push(count.clone());
                    }
                    _ => {
                        count = Integer::from(0);
                        nodes.push(count.clone());
                    }
                }
            }
        }
    }

    count
}

fn count_on_ccg(ccg: &[String], assumptions: &[i32]) -> Integer {
    let ccg_nodes = ccg.iter().collect::<Vec<_>>();
    let node_count = ccg_nodes.len();

    let mut nodes = Vec::with_capacity(node_count);

    let mut count = Integer::from(0);

    for node in ccg_nodes {
        let mut spec = node.split_whitespace();
        match spec.next() {
            Some("*") => {
                let n_children = spec
                    .next()
                    .and_then(|child| usize::from_str(child).ok())
                    .unwrap();

                let children_ = &spec
                    .flat_map(|child| usize::from_str(child).ok())
                    .collect::<Vec<_>>()[..n_children];
                let children = children_
                    .iter()
                    .map(|idx| unsafe { nodes.get_unchecked(*idx) })
                    .collect::<Vec<_>>();

                count = children
                    .iter()
                    .fold(Integer::from(1), |acc, child_val: &&Integer| {
                        acc * &(*child_val).clone()
                    });

                nodes.push(count.clone());
            }
            Some("+") => {
                count = Integer::from(0);
                let n_children = spec
                    .next()
                    .and_then(|child| usize::from_str(child).ok())
                    .unwrap();

                let children_ = &spec
                    .flat_map(|child| usize::from_str(child).ok())
                    .collect::<Vec<_>>()[..n_children];
                let children = children_
                    .iter()
                    .map(|idx| unsafe { nodes.get_unchecked(*idx) })
                    .collect::<Vec<_>>();

                children.iter().for_each(|child_val| {
                    count += &(*child_val).clone();
                });

                nodes.push(count.clone());
            }
            o => {
                let lit = o
                    .and_then(|l| i32::from_str(l).ok())
                    .expect("reading literal failed.");

                count = spec
                    .next()
                    .and_then(|l| i32::from_str(l).map(Integer::from).ok())
                    .expect("reading val failed.");

                match assumptions.contains(&-lit) {
                    false => {
                        nodes.push(count.clone());
                    }
                    _ => {
                        count = Integer::from(0);
                        nodes.push(count.clone());
                    }
                }
            }
        }
    }

    count
}

pub fn anytime_cg_count(
    ccg: impl AsRef<Path>,
    cycles: std::str::Lines,
    assumptions: &[i32],
    depth: usize,
) -> Integer {
    let cycles_file = cycles.collect::<Vec<_>>();

    let ccg_nodes = read_to_string(ccg)
        .unwrap()
        .lines()
        .into_iter()
        .filter(|l| !l.starts_with('c'))
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let mut count = count_on_ccg(&ccg_nodes, assumptions);

    /*
    let ucs = cycles_file
        .iter()
        //.filter(|l| !l.starts_with('c'))
        .map(|l| {
            l.split_whitespace()
                .map(|i| i32::from_str(i).ok())
                .flatten()
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
        */

    #[cfg(not(feature = "prefilter"))]
    let ucs = cycles_file
        .iter()
        .map(|l| {
            l.split_whitespace()
                .map(|i| i32::from_str(i).expect("error: reading ucs failed."))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    #[cfg(feature = "prefilter")]
    let mut n_unfiltered = 0;
    #[cfg(feature = "prefilter")]
    let ucs = cycles_file
        .iter()
        .map(|l| {
            n_unfiltered += 1;
            l.split_whitespace()
                .map(|i| i32::from_str(i).expect("error: reading ucs failed."))
                .collect::<Vec<_>>()
        })
        .filter(|c| !assumptions.iter().any(|l| c.contains(&-l)))
        .collect::<Vec<_>>();

    let mut i = 1;

    let n_cycles = ucs.len();

    #[cfg(not(feature = "prefilter"))]
    let d = if depth == 0 { n_cycles + 1 } else { depth + 1 };
    #[cfg(not(feature = "prefilter"))]
    println!("c o d={:?} n={:?} a={:?}", d - 1, n_cycles, assumptions);

    #[cfg(feature = "prefilter")]
    let d = if depth == 0 || depth > n_cycles {
        n_cycles + 1
    } else {
        depth + 1
    };
    #[cfg(feature = "prefilter")]
    println!(
        "c o d={:?} n={:?} p={:?} a={:?}",
        d - 1,
        n_unfiltered,
        n_cycles,
        assumptions
    );

    if count == 0 {
        println!("c o UNSATISFIABLE");
        return count;
    } else {
        println!("c o 0 {:.2}", count.to_f64().log10());
    }

    let mut prev = count.clone();

    while i < d {
        #[cfg(feature = "seq")]
        let lambda_i = (0..n_cycles).combinations(i);
        #[cfg(not(feature = "seq"))]
        let lambda_i = (0..n_cycles).combinations(i).collect::<Vec<_>>();

        match i % 2 != 0 {
            true =>
            // -
            {
                #[cfg(feature = "seq")]
                for gamma in lambda_i {
                    let mut assumptions_ = gamma
                        .iter()
                        .map(|idx| unsafe { ucs.get_unchecked(*idx) })
                        .fold(vec![], |mut a, v| {
                            a.extend(v);
                            a
                        });
                    assumptions_.extend(assumptions);
                    count -= count_on_ccg(&ccg_nodes, &assumptions_);
                }

                #[cfg(not(feature = "seq"))]
                {
                    let c = lambda_i
                        .par_iter()
                        .map(|gamma| {
                            let mut assumptions_: Vec<i32> = gamma
                                .iter()
                                .map(|idx| unsafe { ucs.get_unchecked(*idx) })
                                .fold(vec![], |mut a, v| {
                                    a.extend(v);
                                    a
                                });
                            assumptions_.extend(assumptions);
                            count_on_ccg(&ccg_nodes, &assumptions_)
                        })
                        .sum::<Integer>();
                    count -= c;
                }
            }
            _ =>
            // +
            {
                #[cfg(feature = "seq")]
                for gamma in lambda_i {
                    let mut assumptions_ = gamma
                        .iter()
                        .map(|idx| unsafe { ucs.get_unchecked(*idx) })
                        .fold(vec![], |mut a, v| {
                            a.extend(v);
                            a
                        });
                    assumptions_.extend(assumptions);
                    count += count_on_ccg(&ccg_nodes, &assumptions_);
                }

                #[cfg(not(feature = "seq"))]
                {
                    let c = lambda_i
                        .par_iter()
                        .map(|gamma| {
                            let mut assumptions_: Vec<i32> = gamma
                                .iter()
                                .map(|idx| unsafe { ucs.get_unchecked(*idx) })
                                .fold(vec![], |mut a, v| {
                                    a.extend(v);
                                    a
                                });
                            assumptions_.extend(assumptions);
                            count_on_ccg(&ccg_nodes, &assumptions_)
                        })
                        .sum::<Integer>();
                    count += c;
                }
            }
        }

        let prevl10 = prev.clone().abs().to_f64().log10();
        let countl10 = count.clone().abs().to_f64().log10();
        let delta = (prevl10 - countl10).abs();
        //println!("c o delta {:?} {:?} {:?} {:.2}", i, prev, count, delta);
        if delta.is_nan() {
            println!("c o {:?} 0", i);
        } else {
            println!("c o {:?} {:.2}", i, delta);
        }
        if prev == count {
            break;
        } else {
            prev = count.clone()
        }

        i += 1;
    }

    //i -= 1;
    if i % 2 == 0 {
        println!("c o {:.2}+", i as f32 / n_cycles as f32);
    } else {
        println!("c o {:.2}-", i as f32 / n_cycles as f32);
    }

    count
}
