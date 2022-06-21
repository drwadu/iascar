use crate::utils::ToHashSet;
use rayon::prelude::*;
use rug::Integer;
use std::collections::HashMap;
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

#[allow(unused_mut)]
pub fn count_on_cg_with_cycles(
    ccg: impl AsRef<Path>,
    cycles: std::str::Lines,
    assumptions: &[i32],
    mut depth: usize,
) -> Integer {
    let cycles_file = cycles.collect::<Vec<_>>();
    let cycles_mappings = cycles_file.iter().filter(|l| l.starts_with("c "));
    let mut ccg_mappings: HashMap<String, i32> = HashMap::new();
    for m in read_to_string(&ccg)
        .unwrap_or_else(|_| "".to_string())
        .lines()
        .filter(|l| l.starts_with("c "))
    {
        if !m.is_empty() {
            let mut line = m.split_whitespace().skip(1);
            let s = line.next().unwrap();
            let i = line.next().and_then(|i| i32::from_str(i).ok()).unwrap(); // TODO
            ccg_mappings.insert(s.to_string(), i);
        }
    }
    let mut mappings: HashMap<i32, i32> = HashMap::new();
    for m in cycles_mappings {
        let mut line = m.split_whitespace().skip(1);
        let s = line.next().unwrap();
        let i = line.next().and_then(|i| i32::from_str(i).ok()).unwrap(); // TODO
        let j = ccg_mappings.get(s).unwrap();
        mappings.insert(i, *j);
    }

    let ccg_nodes = read_to_string(ccg)
        .unwrap()
        .lines()
        .into_iter()
        .filter(|l| !l.starts_with('c'))
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let mut count = count_on_ccg(&ccg_nodes, assumptions);

    if depth > 0 {
        #[cfg(feature = "no_undercounting")]
        #[allow(unused_assignments)]
        {
            if depth % 2 != 0 {
                if depth == 1 {
                    depth += 1;
                } else {
                    depth -= 1;
                }
            }
        }

        // if in first alternation, then overlaps of two exlusion routes (inclusion routes of size two)
        // take routes until alternation depth
        let mut s = 0;
        let mut m = 0;
        let mut routes = vec![];
        for l in cycles_file.iter().skip(1).filter(|l| !l.starts_with('c')) {
            if m == depth {
                break;
            }
            match l.starts_with('m') {
                true => {
                    routes.push((
                        0,
                        l.split(' ')
                            .skip(1)
                            .filter_map(|i| {
                                i32::from_str(i)
                                    .map(|l| {
                                        if l < 0 {
                                            mappings.get(&l.abs()).map(|j| -j)
                                        } else {
                                            mappings.get(&l).copied()
                                        }
                                    })
                                    .ok()
                            })
                            .flatten()
                            .collect::<Vec<_>>(),
                    ));
                    let s_ = 0;
                    if s != s_ {
                        m += 1
                    }
                    s = s_;
                }
                _ => {
                    routes.push((
                        1,
                        l.split(' ')
                            .skip(1)
                            .filter_map(|i| {
                                i32::from_str(i)
                                    .map(|l| {
                                        if l < 0 {
                                            mappings.get(&l.abs()).map(|j| -j)
                                        } else {
                                            mappings.get(&l).copied()
                                        }
                                    })
                                    .ok()
                            })
                            .flatten()
                            .collect::<Vec<_>>(),
                    ));
                    let s_ = 1;
                    if s != s_ {
                        m += 1
                    }
                    s = s_;
                }
            }
        }

        #[cfg(not(feature = "sequential_early_termination"))]
        {
            // par_iter preserves order: https://github.com/rayon-rs/rayon/issues/551
            let counts = routes
                .par_iter()
                .map(|(s, r)| {
                    let mut delta = assumptions.to_vec().clone();
                    delta.extend(r);
                    (s, count_on_ccg(&ccg_nodes, &delta))
                })
                .collect::<Vec<_>>();

            for (s, c) in counts {
                if *s == 0 {
                    count -= c;
                } else {
                    count += c;
                }
            }
        }

        #[cfg(feature = "sequential_early_termination")]
        {
            let mut count_ = Integer::from(0);
            let mut s_ = &0;
            for (s, r) in routes.iter() {
                let mut delta = assumptions.to_vec().clone();
                delta.extend(r);
                let a = count_on_ccg(&ccg_nodes, &delta);

                if s_ != s && count_ == count {
                    break;
                } else {
                    count_ = count.clone();
                    s_ = s;
                }

                if *s == 0 {
                    count -= a;
                } else {
                    count += a;
                }
            }
        }
    } else {
        let mut ms = vec![];
        let mut ps = vec![];
        cycles_file
            .iter()
            .skip(1)
            .filter(|l| !l.starts_with('c'))
            .for_each(|l| match l.starts_with('m') {
                true => ms.push(
                    l.split(' ')
                        .skip(1)
                        .filter_map(|i| {
                            i32::from_str(i)
                                .map(|l| {
                                    if l < 0 {
                                        mappings.get(&l.abs()).map(|j| -j)
                                    } else {
                                        mappings.get(&l).copied()
                                    }
                                })
                                .ok()
                        })
                        .flatten()
                        .collect::<Vec<_>>(),
                ),
                _ => ps.push(
                    l.split(' ')
                        .skip(1)
                        .filter_map(|i| {
                            i32::from_str(i)
                                .map(|l| {
                                    if l < 0 {
                                        mappings.get(&l.abs()).map(|j| -j)
                                    } else {
                                        mappings.get(&l).copied()
                                    }
                                })
                                .ok()
                        })
                        .flatten()
                        .collect::<Vec<_>>(),
                ),
            });

        #[cfg(not(feature = "sequential_early_termination"))]
        {
            // parallel order-ignoring
            count -= ms
                .par_iter()
                .map(|m| {
                    let mut delta = assumptions.to_vec().clone();
                    delta.extend(m);
                    count_on_ccg(&ccg_nodes, &delta)
                })
                .sum::<Integer>();
            count += ps
                .par_iter()
                .map(|p| {
                    let mut delta = assumptions.to_vec().clone();
                    delta.extend(p);
                    count_on_ccg(&ccg_nodes, &delta)
                })
                .sum::<Integer>();
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

pub fn count_on_cg(filename: impl AsRef<Path>, assumptions: &[i32]) -> Integer {
    let nnf = read_to_string(&filename).unwrap_or_else(|_| "".to_string());

    let mut lines = nnf.lines().filter(|l| !l.starts_with("c "));

    if assumptions.is_empty() {
        return lines
            .next()
            .and_then(|stats| stats.split_whitespace().last())
            .and_then(|c| f32::from_str(c).ok())
            .map(|l| 10f32.powf(l).round() as u128)
            .map(Integer::from)
            .expect("cannot parse count.");
    }

    let node_count = lines
        .next()
        .and_then(|stats| stats.split_whitespace().nth(1))
        .and_then(|s| usize::from_str(s).ok())
        .expect("reading node count failed.");

    let mut nodes = Vec::with_capacity(node_count);

    let mut count = Integer::from(0);

    lines.for_each(|line| {
        let mut spec = line.split_whitespace();
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
                    .unwrap(); // TODO

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
    });

    count
}
