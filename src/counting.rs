use crate::utils::ToHashSet;
use crate::{AND, OR};
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

pub fn count_on_cg_with_cycles(
    ccg: impl AsRef<Path>,
    cycles: std::str::Lines,
    assumptions: &[i32],
    depth: usize,
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
        // if in first alternation, then overlaps of two exlusion routes (inclusion routes of size two)
        // take routes until alternation depth
        let mut s = 0;
        let mut m = 0;
        let mut routes = vec![];
        for l in cycles_file.iter().skip(1).filter(|l| !l.starts_with('c')) {
            if m == depth + 1 {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn supported_8_queens() {
        // q(1, 1)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[52]), 4);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-52]), 88);

        // q(2, 1)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[61]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-61]), 84);

        // q(3, 1)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[69]), 16);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-69]), 76);

        // q(4, 1)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[76]), 18);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-76]), 74);

        // q(5, 1)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[82]), 18);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-82]), 74);

        // q(6, 1)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[87]), 16);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-87]), 76);

        // q(7, 1)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[91]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-91]), 84);

        // q(8, 1)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[101]), 4);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-101]), 88);

        // q(1, 2)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[44]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-44]), 84);

        // q(2, 2)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[53]), 16);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-53]), 76);

        // q(3, 2)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[62]), 14);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-62]), 78);

        // q(4, 2)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[70]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-70]), 84);

        // q(5, 2)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[77]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-77]), 84);

        // q(6, 2)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[83]), 14);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-83]), 78);

        // q(7, 2)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[88]), 16);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-88]), 76);

        // q(8, 2)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[92]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-92]), 84);

        // q(1, 3)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[37]), 16);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-37]), 76);

        // q(2, 3)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[45]), 14);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-45]), 78);

        // q(3, 3)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[54]), 4);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-54]), 88);

        // q(4, 3)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[63]), 12);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-63]), 80);

        // q(5, 3)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[71]), 12);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-71]), 80);

        // q(6, 3)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[78]), 4);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-78]), 88);

        // q(7, 3)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[84]), 14);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-84]), 78);

        // q(8, 3)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[89]), 16);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-89]), 76);

        // q(1, 4)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[31]), 18);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-31]), 74);

        // q(2, 4)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[38]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-38]), 84);

        // q(3, 4)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[46]), 12);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-46]), 80);

        // q(4, 4)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[55]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-55]), 84);

        // q(5, 4)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[64]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-64]), 84);

        // q(6, 4)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[72]), 12);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-72]), 80);

        // q(7, 4)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[79]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-79]), 84);

        // q(8, 4)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[85]), 18);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-85]), 74);

        // q(1, 5)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[26]), 18);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-26]), 74);

        // q(2, 5)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[32]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-32]), 84);

        // q(3, 5)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[39]), 12);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-39]), 80);

        // q(4, 5)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[47]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-47]), 84);

        // q(5, 5)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[56]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-56]), 84);

        // q(6, 5)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[65]), 12);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-65]), 80);

        // q(7, 5)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[73]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-73]), 84);

        // q(8, 5)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[80]), 18);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-80]), 74);

        // q(1, 6)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[22]), 16);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-22]), 76);

        // q(2, 6)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[27]), 14);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-27]), 78);

        // q(3, 6)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[33]), 4);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-33]), 88);

        // q(4, 6)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[40]), 12);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-40]), 80);

        // q(5, 6)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[48]), 12);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-48]), 80);

        // q(6, 6)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[57]), 4);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-57]), 88);

        // q(7, 6)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[66]), 14);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-66]), 78);

        // q(8, 6)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[74]), 16);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-74]), 76);

        // q(1, 7)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[19]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-19]), 84);

        // q(2, 7)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[23]), 16);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-23]), 76);

        // q(3, 7)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[28]), 14);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-28]), 78);

        // q(4, 7)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[34]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-34]), 84);

        // q(5, 7)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[41]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-41]), 84);

        // q(6, 7)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[49]), 14);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-49]), 78);

        // q(7, 7)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[58]), 16);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-58]), 76);

        // q(8, 7)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[67]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-67]), 84);

        // q(1, 8)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[100]), 4);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-100]), 88);

        // q(2, 8)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[20]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-20]), 84);

        // q(3, 8)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[24]), 16);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-24]), 76);

        // q(4, 8)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[29]), 18);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-29]), 74);

        // q(5, 8)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[35]), 18);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-35]), 74);

        // q(6, 8)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[42]), 16);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-42]), 76);

        // q(7, 8)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[50]), 8);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-50]), 84);

        // q(8, 8)
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[59]), 4);
        assert_eq!(count_on_cg("test_instances/8_queens.ccg", &[-59]), 88);
    }

    #[test]
    fn supported_grid() {
        (29..109).for_each(|assumption| {
            assert_eq!(
                count_on_cg("test_instances/grid.ccg", &[assumption]),
                40_320
            );
            assert_eq!(
                count_on_cg("test_instances/grid.ccg", &[-assumption]),
                322_560
            );
        });

        // set_obj_cell(x,y) s.t. x = y
        assert_eq!(
            count_on_cg(
                "test_instances/grid.ccg",
                &[29, 39, 49, 59, 69, 79, 89, 99, 109]
            ),
            1
        );
    }

    #[test]
    fn supported_af_st() {
        // in(a48)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[672]), 1400);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-672]), 6296);

        // in(a84)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[480]), 972);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-480]), 6724);

        // in(a28)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[602]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-602]), 6088);

        // in(a22)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[584]), 2304);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-584]), 5392);

        // in(a67)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[746]), 6256);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-746]), 1440);

        // in(a140)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[542]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-542]), 6088);

        // in(a51)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[682]), 3232);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-682]), 4464);

        // in(a142)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[552]), 2136);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-552]), 5560);

        // in(a65)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[742]), 2136);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-742]), 5560);

        // in(a6)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[762]), 6896);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-762]), 800);

        // in(a161)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[648]), 1656);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-648]), 6040);

        // in(a11)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[538]), 5560);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-538]), 2136);

        // in(a14)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[560]), 2136);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-560]), 5560);

        // in(a123)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[486]), 5200);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-486]), 2496);

        // in(a25)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[600]), 6088);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-600]), 1608);

        // in(a70)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[772]), 2304);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-772]), 5392);

        // in(a68)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[744]), 1440);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-744]), 6256);

        // in(a72)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[776]), 1440);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-776]), 6256);

        // in(a73)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[774]), 544);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-774]), 7152);

        // in(a61)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[728]), 1400);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-728]), 6296);

        // in(a158)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[696]), 972);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-696]), 6724);

        // in(a139)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[612]), 972);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-612]), 6724);

        // in(a147)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[582]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-582]), 6088);

        // in(a117)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[516]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-516]), 6088);

        // in(a126)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[468]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-468]), 6088);

        // in(a80)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[466]), 2136);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-466]), 5560);

        // in(a129)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[478]), 6448);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-478]), 1248);

        // in(a91)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[506]), 2136);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-506]), 5560);

        // in(a118)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[520]), 1400);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-520]), 6296);

        // in(a12)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[554]), 6896);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-554]), 800);

        // in(a19)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[566]), 6088);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-566]), 1608);

        // in(a38)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[646]), 5560);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-646]), 2136);

        // in(a136)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[624]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-624]), 6088);

        // in(a152)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[662]), 1864);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-662]), 5832);

        // in(a64)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[732]), 2304);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-732]), 5392);

        // in(a3)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[756]), 2448);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-756]), 5248);

        // in(a132)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[596]), 1448);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-596]), 6248);

        // in(a24)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[590]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-590]), 6088);

        // in(a137)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[608]), 4952);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-608]), 2744);

        // in(a34)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[636]), 6724);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-636]), 972);

        // in(a111)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[524]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-524]), 6088);

        // in(a10)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[544]), 2136);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-544]), 5560);

        // in(a93)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[514]), 3848);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-514]), 3848);

        // in(a148)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[570]), 972);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-570]), 6724);

        // in(a33)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[626]), 6724);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-626]), 972);

        // in(a75)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[778]), 1448);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-778]), 6248);

        // in(a121)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[464]), 1440);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-464]), 6256);

        // in(a71)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[770]), 660);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-770]), 7036);

        // in(a112)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[528]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-528]), 6088);

        // in(a104)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[714]), 1440);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-714]), 6256);

        // in(a59)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[706]), 3848);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-706]), 3848);

        // in(a76)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[456]), 1400);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-456]), 6296);

        // in(a57)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[702]), 3848);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-702]), 3848);

        // in(a9)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[768]), 800);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-768]), 6896);

        // in(a26)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[598]), 6088);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-598]), 1608);

        // in(a88)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[496]), 2136);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-496]), 5560);

        // in(a115)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[508]), 2136);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-508]), 5560);

        // in(a125)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[494]), 800);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-494]), 6896);

        // in(a50)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[686]), 1400);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-686]), 6296);

        // in(a31)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[618]), 2136);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-618]), 5560);

        // in(a49)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[676]), 6296);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-676]), 1400);

        // in(a16)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[564]), 1440);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-564]), 6256);

        // in(a99)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[540]), 800);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-540]), 6896);

        // in(a122)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[482]), 1440);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-482]), 6256);

        // in(a1)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[752]), 6296);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-752]), 1400);

        // in(a4)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[758]), 1400);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-758]), 6296);

        // in(a124)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[490]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-490]), 6088);

        // in(a23)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[594]), 1864);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-594]), 5832);

        // in(a135)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[620]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-620]), 6088);

        // in(a154)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[670]), 1440);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-670]), 6256);

        // in(a56)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[704]), 3848);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-704]), 3848);

        // in(a5)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[760]), 800);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-760]), 6896);

        // in(a55)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[698]), 6448);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-698]), 1248);

        // in(a108)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[710]), 544);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-710]), 7152);

        // in(a157)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[692]), 944);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-692]), 6752);

        // in(a146)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[580]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-580]), 6088);

        // in(a97)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[530]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-530]), 6088);

        // in(a17)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[562]), 2744);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-562]), 4952);

        // in(a60)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[718]), 3848);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-718]), 3848);

        // in(a150)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[678]), 1440);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-678]), 6256);

        // in(a130)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[588]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-588]), 6088);

        // in(a116)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[512]), 972);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-512]), 6724);

        // in(a98)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[546]), 1400);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-546]), 6296);

        // in(a109)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[712]), 972);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-712]), 6724);

        // in(a66)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[740]), 1440);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-740]), 6256);

        // in(a15)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[558]), 6256);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-558]), 1440);

        // in(a52)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[694]), 1248);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-694]), 6448);

        // in(a29)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[606]), 4648);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-606]), 3048);

        // in(a156)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[688]), 800);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-688]), 6896);

        // in(a47)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[674]), 800);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-674]), 6896);

        // in(a102)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[734]), 3848);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-734]), 3848);

        // in(a95)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[522]), 2136);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-522]), 5560);

        // in(a13)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[550]), 800);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-550]), 6896);

        // in(a92)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[518]), 3848);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-518]), 3848);

        // in(a37)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[638]), 2136);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-638]), 5560);

        // in(a160)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[644]), 1440);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-644]), 6256);

        // in(a54)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[700]), 1248);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-700]), 6448);

        // in(a74)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[780]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-780]), 6088);

        // in(a144)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[574]), 3848);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-574]), 3848);

        // in(a43)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[660]), 6088);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-660]), 1608);

        // in(a62)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[724]), 1400);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-724]), 6296);

        // in(a2)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[754]), 5248);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-754]), 2448);

        // in(a86)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[488]), 972);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-488]), 6724);

        // in(a85)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[492]), 1440);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-492]), 6256);

        // in(a35)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[634]), 972);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-634]), 6724);

        // in(a96)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[534]), 800);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-534]), 6896);

        // in(a143)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[556]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-556]), 6088);

        // in(a127)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[470]), 1600);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-470]), 6096);

        // in(a46)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[664]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-664]), 6088);

        // in(a81)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[476]), 1400);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-476]), 6296);

        // in(a36)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[640]), 972);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-640]), 6724);

        // in(a32)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[630]), 972);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-630]), 6724);

        // in(a18)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[568]), 4952);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-568]), 2744);

        // in(a119)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[504]), 660);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-504]), 7036);

        // in(a153)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[666]), 5392);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-666]), 2304);

        // in(a113)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[532]), 1400);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-532]), 6296);

        // in(a20)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[578]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-578]), 6088);

        // in(a114)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[536]), 944);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-536]), 6752);

        // in(a138)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[610]), 2744);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-610]), 4952);

        // in(a151)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[658]), 5392);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-658]), 2304);

        // in(a133)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[614]), 972);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-614]), 6724);

        // in(a21)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[586]), 3048);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-586]), 4648);

        // in(a94)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[526]), 3848);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-526]), 3848);

        // in(a41)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[654]), 2136);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-654]), 5560);

        // in(a0)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[750]), 1400);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-750]), 6296);

        // in(a42)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[652]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-652]), 6088);

        // in(a69)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[748]), 3048);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-748]), 4648);

        // in(a103)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[738]), 2136);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-738]), 5560);

        // in(a77)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[454]), 3848);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-454]), 3848);

        // in(a141)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[548]), 1440);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-548]), 6256);

        // in(a27)
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[604]), 1608);
        assert_eq!(count_on_cg("test_instances/af_st.ccg", &[-604]), 6088);
    }

    #[test]
    fn stable_ce() {
        // {4,2,5} {3}
        // {4,3,5}
        // depth bounded by 1
        let f = read_to_string("test_instances/ce.cycles").expect("provide test instance.");
        let mut lines = f.lines();
        lines.next();
        assert_eq!(
            count_on_cg_with_cycles("test_instances/ce.ccg", lines.clone(), &[4], 0,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/ce.ccg", lines.clone(), &[-4], 0,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/ce.ccg", lines.clone(), &[3], 0,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/ce.ccg", lines.clone(), &[-3], 0,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/ce.ccg", lines.clone(), &[5], 0,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/ce.ccg", lines.clone(), &[-5], 0,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/ce.ccg", lines.clone(), &[2], 0,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/ce.ccg", lines.clone(), &[-2], 0,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/ce.ccg", lines.clone(), &[2, -5], 0,),
            0
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/ce.ccg", lines.clone(), &[-2, 3], 0,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/ce.ccg", lines.clone(), &[3, 4], 0,),
            0
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/ce.ccg", lines.clone(), &[-3, 4], 0,),
            1
        );
    }

    #[test]
    fn stable_raki1() {
        // {2,6,7,4,8} {3,6,4,8} {2,7,6,5,8} {7,3,5}
        // {3,7,6,5,8}
        // depth bounded by 2
        let f = read_to_string("test_instances/raki1.cycles").expect("provide test instance.");
        let mut lines = f.lines();
        lines.next();

        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[2], 0,),
            2
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[2], 1,),
            2
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[-2], 0,),
            2
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[-2], 1,),
            2
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[3], 0,),
            2
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[3], 1,),
            2
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[-3], 0,),
            2
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[-3], 1,),
            2
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[7], 0,),
            3
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[7], 1,),
            3
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[-7], 0,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[-7], 1,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[6], 0,),
            3
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[6], 1,),
            3
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[-6], 0,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[-6], 1,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[4], 0,),
            2
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[4], 1,),
            2
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[-4], 0,),
            2
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[-4], 1,),
            2
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[8], 0,),
            3
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[8], 1,),
            3
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[-8], 0,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[-8], 1,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[2, 3], 0,),
            0
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[2, 3], 1,),
            0
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[2, -3], 0,),
            2
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[2, -3], 1,),
            2
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[-2, 7], 0,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[-2, 7], 1,),
            1
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[7, 6], 0,),
            2
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[7, 6], 1,),
            2
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[-7, -6], 0,),
            0
        );
        assert_eq!(
            count_on_cg_with_cycles("test_instances/raki1.ccg", lines.clone(), &[-7, -6], 1,),
            0
        );
    }
}
