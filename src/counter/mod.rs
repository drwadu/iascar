#[cfg(not(feature = "seq"))]
use rayon::prelude::*;
use rug::Integer;
use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::path::Path;
use std::str::FromStr;

use crate::{SAND, SOR};

#[derive(Debug, Clone)]
pub enum CounterError {
    ReadingError(String),
}

pub type Result<T> = std::result::Result<T, CounterError>;

#[derive(Debug, Clone)]
pub struct Counter {
    counting_graph: Vec<String>,
    mapping: HashMap<String, i32>,
    node_count: usize,
    overall_count: f64,
}
impl Counter {
    pub fn new(ccg_filename: impl AsRef<Path>) -> Result<Self> {
        let dag = read_to_string(&ccg_filename)
            .map_err(|err| CounterError::ReadingError(err.to_string()))?;

        let mut lines = dag.lines();
        let (overall_count, node_count) = lines
            .next()
            .and_then(|line| {
                let mut ls = line.split_whitespace();
                ls.nth(1).zip(ls.last())
            })
            .and_then(|(nc, oc)| f64::from_str(oc).ok().zip(usize::from_str(nc).ok()))
            .ok_or(CounterError::ReadingError(
                "could not read node count.".to_owned(),
            ))?;

        let mut mapping = HashMap::new();
        let mut atom_count = 0;
        for l in lines.take_while(|line| line.starts_with("c ")) {
            let mut iter = l.split_whitespace();
            iter.next();
            let v = iter.next().and_then(|s| i32::from_str(s).ok()).ok_or(
                CounterError::ReadingError("could not read integer in mapping.".to_owned()),
            )?;
            let k = iter.next().ok_or(CounterError::ReadingError(
                "could not read atom in mapping.".to_owned(),
            ))?;
            mapping.insert(k.to_owned(), v);
            atom_count += 1;
        }

        let mut counting_graph = Vec::with_capacity(node_count);
        dag.lines()
            .skip(atom_count + 1)
            .for_each(|node| counting_graph.push(node.to_owned()));

        Ok(Self {
            counting_graph,
            mapping,
            node_count,
            overall_count,
        })
    }

    pub fn count<S: ToString>(&self, assume: impl Iterator<Item = S>) -> Integer {
        let mut count = Integer::from(0);

        let assumptions = assume
            .map(|s| self.read_assumption(s.to_string()))
            .flatten()
            .collect::<Vec<_>>();

        let mut nodes = Vec::with_capacity(self.node_count);

        self.counting_graph.iter().for_each(|node| {
            let mut spec = node.split_whitespace();
            match spec.next() {
                Some(SAND) => {
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
                Some(SOR) => {
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
        });

        count
    }

    pub fn overall_count(&self) -> f64 {
        self.overall_count
    }

    pub fn facet_count<S: ToString>(&self, assume: impl Iterator<Item = S>) -> usize {
        let mut curr = self.mapping.keys().collect::<HashSet<_>>();

        let mut base_assumptions = vec![];
        assume.for_each(|s| {
            let str = s.to_string();
            curr.remove(&str); // NOTE: assumption is inc/exc facet
            curr.remove(&str[1..].to_owned()); // NOTE: assumption is inc/exc facet
            base_assumptions.push(str);
        });

        curr.into_iter()
            .filter(|a| {
                let str = a.to_string();
                base_assumptions.push(format!("~{str}"));
                let cond = if self.count(base_assumptions.iter()) > 0 {
                    base_assumptions.pop();
                    base_assumptions.push(str.clone());
                    let p = self.count(base_assumptions.iter()) > 0;
                    base_assumptions.pop();
                    p
                } else {
                    base_assumptions.pop();
                    false
                };

                cond
            })
            .count()
    }

    fn read_assumption(&self, assumption: String) -> Option<i32> {
        match assumption.starts_with("~") {
            true => self
                .mapping
                .get(&assumption[1..].to_owned())
                .and_then(|i| Some(-i)),
            _ => self.mapping.get(&assumption).copied(),
        }
    }
}

pub fn log10_count(count: Integer) -> f64 {
    count.to_f64().log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_tiny() -> Result<()> {
        let counter = Counter::new("example.ccg")?;
        println!("{:?}", counter.count(["a"].iter()));
        println!("{:?}", counter.count(["~a"].iter()));
        println!("{:?}", counter.facet_count(["a"].iter()));
        println!("{:?}", counter.facet_count(["~a"].iter()));
        Ok(())
    }

    #[test]
    fn count_8q() -> Result<()> {
        let counter = Counter::new("8_queens.ccg")?;
        println!("{:?}", counter.count(["q(1,1)"].iter()));
        println!("{:?}", counter.count(["~q(1,1)"].iter()));
        println!("{:?}", counter.facet_count(["q(1,1)"].iter()));
        println!("{:?}", counter.facet_count(["~q(1,1)"].iter()));
        Ok(())
    }
}
