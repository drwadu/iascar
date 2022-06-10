use crate::{AND, OR};
use clingo::{Control, Literal, Part};
use rug::Integer;
use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::io::Write;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranspilerError {
    #[error("clingo error")]
    Clingo(#[from] clingo::ClingoError),
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("unwrapped None")]
    None,
}

pub type Result<T> = std::result::Result<T, TranspilerError>;

pub fn transpile(nnf_file: String) -> Result<()> {
    let name = nnf_file.split('.').next().ok_or(TranspilerError::None)?;
    let lp_path = &format!("{}.lp", name);

    let cnf_mappings = read_cnf_mappings(name)?;

    let lp = read_to_string(lp_path)?;

    let mut ctl = Control::new(vec!["0".to_owned()])?;

    ctl.add("base", &[], &lp)
        .and_then(|_| Part::new("base", &[]))
        .and_then(|p| ctl.ground(&[p]))?;

    let mut atom_mappings: HashMap<i32, Literal> = HashMap::new();

    let mut supported_atoms = vec![];

    for atom in ctl.symbolic_atoms()?.iter()? {
        let atom_as_string = atom.symbol().map(|s| s.to_string())??;
        let atom_as_lit = atom.literal()?;

        [cnf_mappings.get(&atom_as_string)]
            .iter()
            .flatten()
            .for_each(|cnf_atom_int| {
                atom_mappings.insert(**cnf_atom_int, atom_as_lit);
            });
        supported_atoms.push(atom_as_string);
    }

    #[allow(clippy::needless_collect)]
    let falsified_by_gringo = cnf_mappings
        .keys()
        .filter(|a| !supported_atoms.contains(a))
        .map(|a| {
            let i = *cnf_mappings.get(a).unwrap_or(&0);
            i
        })
        .collect::<Vec<_>>();

    let nnf = read_to_string(&nnf_file).unwrap_or_else(|_| "".to_string());

    let mut lines = nnf.lines();
    let node_count = lines
        .next()
        .map(|stats| {
            stats
                .split_whitespace()
                .skip(1)
                .flat_map(usize::from_str)
                .collect::<Vec<_>>()
        })
        .map(|xs| xs[0])
        .ok_or(TranspilerError::None)?;

    let mut nodes = Vec::with_capacity(node_count);

    let mut n_nodes = 0;
    let mut n_edges = 0;

    let mut atom_popped_ids: HashSet<usize> = HashSet::new();
    let mut popped_ids: HashSet<usize> = HashSet::new();
    let mut new_vars_count = 0;
    let mut node_id_diffs = HashMap::<usize, usize>::new();
    let mut n_popped = 0;

    for (i, line) in lines.enumerate() {
        let mut spec = line.split_whitespace();

        match spec.next() {
            Some("L") => {
                let lit = spec
                    .next()
                    .and_then(|l| i32::from_str(l).ok())
                    .ok_or(TranspilerError::None)?;

                let atom = lit.abs();

                if !(atom_mappings.get(&atom).is_some() || falsified_by_gringo.contains(&atom)) {
                    nodes.push(vec![]);
                    atom_popped_ids.insert(i);
                    n_popped += 1;
                } else {
                    let lit_int = Integer::from(lit);
                    let mut node = Vec::with_capacity(2);

                    n_nodes += 1;
                    if lit > 0 {
                        new_vars_count += 1;
                    }

                    let val = Integer::from(1);

                    node.push(val);
                    node.push(lit_int);
                    nodes.push(node);

                    node_id_diffs.insert(i, i - n_popped);
                }
            }
            gate => {
                let is_or_node = gate == Some("O");

                if is_or_node {
                    spec.next();
                }

                let children = spec
                    .skip(1)
                    .filter_map(|nnf_child_id| usize::from_str(nnf_child_id).ok())
                    .filter(|nnf_child_id| !atom_popped_ids.contains(nnf_child_id))
                    .map(|nnf_child_id| {
                        (
                            nnf_child_id,
                            *node_id_diffs.get(&nnf_child_id).unwrap_or(&nnf_child_id),
                        )
                    })
                    .filter(|(_, cgg_child_id)| *cgg_child_id < n_nodes)
                    .map(|(nnf_child_id, ccg_child_id)| {
                        (
                            nnf_child_id,
                            unsafe { nodes.get_unchecked(nnf_child_id) },
                            ccg_child_id,
                        )
                    })
                    .collect::<Vec<_>>();

                let n_children = children.len();

                let mut node = Vec::with_capacity(n_children + 2);

                match n_children {
                    0 => {
                        popped_ids.insert(i);
                        node_id_diffs.insert(i, node_count); // TODO
                        n_popped += 1;
                    }
                    1 => {
                        let is_root = i == node_count - 1;
                        if !is_root {
                            let (nnf_child_node, ccg_child_id) = {
                                let t = unsafe { children.get_unchecked(0) };
                                (t.1.clone(), t.2)
                            };
                            node = nnf_child_node;
                            node_id_diffs.insert(i, ccg_child_id);
                        }
                        popped_ids.insert(i);
                        n_popped += 1;
                    }
                    n => {
                        n_nodes += 1;
                        n_edges += n;

                        if !is_or_node {
                            let val =
                                children
                                    .iter()
                                    .fold(Integer::from(1), |acc, (_, child, _)| {
                                        acc * match child.get(0) {
                                            // TODO
                                            Some(v) => v.clone(),
                                            _ => Integer::from(1), // child removed => neutral element
                                        }
                                    });
                            node.push(val);
                            node.extend(
                                children
                                    .iter()
                                    .map(|(_, _, ccg_child_id)| Integer::from(*ccg_child_id)),
                            );
                            node.push(Integer::from(AND));
                        } else {
                            let mut val = Integer::from(0);
                            children.iter().for_each(|(_, child, _)| {
                                val += match child.get(0) {
                                    // TODO
                                    Some(v) => v.clone(),
                                    _ => Integer::from(0), // child removed => neutral element
                                };
                            });
                            node.push(val);
                            node.extend(
                                children
                                    .iter()
                                    .map(|(_, _, ccg_child_id)| Integer::from(*ccg_child_id)),
                            );
                            node.push(Integer::from(OR));
                        }

                        node_id_diffs.insert(i, i - n_popped);
                    }
                }

                nodes.push(node);
            }
        }
    }

    let transpilation = nodes
        .iter()
        .enumerate()
        .filter(|(i, _)| !atom_popped_ids.contains(i) && !popped_ids.contains(i))
        .collect::<Vec<_>>();

    let node_count_t = transpilation.len();

    assert_eq!(node_count_t, node_count - n_popped);
    assert_eq!(node_count_t, n_nodes);

    let root = unsafe { transpilation.get_unchecked(node_count_t - 1) }.1;
    let count = unsafe { root.get_unchecked(0) };

    let stats = format!(
        "ccg {:?} {:?} {:?} {:?}",
        node_count_t,
        n_edges,
        new_vars_count,
        count.to_f32().log10()
    );

    write(&stats, &transpilation, &cnf_mappings)
}

fn read_cnf_mappings(filename: &str) -> Result<HashMap<String, i32>> {
    let cnf_path = &format!("{}.cnf", filename);

    let mut mappings: HashMap<String, i32> = HashMap::new();

    let cnf = read_to_string(cnf_path)?;

    for mapping in cnf.lines().skip(1).filter(|line| line.starts_with('c')) {
        let mut line = mapping.split_whitespace().skip(1);
        let i = line
            .next()
            .and_then(|i| i32::from_str(i).ok())
            .ok_or(TranspilerError::None)?;
        let s = line.next().ok_or(TranspilerError::None)?;

        mappings.insert(s.to_owned(), i);
    }

    Ok(mappings)
}

fn write(
    stats: &str,
    transpilation: &[(usize, &Vec<rug::Integer>)],
    cnf_mappings: &HashMap<String, i32>,
) -> Result<()> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    handle.write_all(format!("{}\n", stats).as_bytes())?;

    for (atom, int) in cnf_mappings {
        handle.write_all(format!("c {} {:?}\n", atom, int).as_bytes())?;
    }

    for (_, node) in transpilation {
        match node.len() {
            2 => {
                for x in node.iter().rev() {
                    handle.write_all(format!("{:?} ", x).as_bytes())?;
                }

                handle.write_all(b"\n")?;
            }
            n => {
                let children_count = n - 2;
                let last_idx = n - 1;

                let kind = match *unsafe { node.get_unchecked(last_idx) } > 0 {
                    true => "*",
                    _ => "+",
                };

                handle.write_all(kind.as_bytes())?;
                handle.write_all(format!(" {:?}", children_count).as_bytes())?;

                for x in &node[1..last_idx] {
                    handle.write_all(format!(" {:?}", x).as_bytes())?;
                }

                #[cfg(feature = "withvals")]
                {
                    let val = unsafe { node.get_unchecked(0) };
                    handle.write_all(format!(" {:?}", val).as_bytes())?;
                }

                handle.write_all(b"\n")?;
            }
        };
    }

    Ok(())
}
