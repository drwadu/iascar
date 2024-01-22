[![Crates.io](https://img.shields.io/crates/v/iascar?label=crates.io%20%28bin%29)](https://crates.io/crates/iascar)
![build workflow](https://github.com/drwadu/iascar/actions/workflows/build.yml/badge.svg)
# iascar - Incremental Answer Set Counter with Anytime Refinement


**iascar** is a propositional model counter tailored toward frequent counting
under assumptions. The counter operates on _smooth deterministic decomposable
negation normal forms_ (sd-DNNFs) or so called _compressed counting graphs_
(CCGs).

## install & build
* Install iascar via cargo `cargo install iascar`
* by default iascar runs in parallel for incremental counting with anytime refinement; to avoid parallel execution, clone this repo and build iascar with `cargo build --release --features seq`
## quickstart
1. download [lp2*-tools](https://research.ics.aalto.fi/software/asp/download/) and [c2d](http://reasoning.cs.ucla.edu/c2d/)
2. set the paths to the respective tools in [build_nnf.sh](iascar/build_nnf.sh), which builds cnfs and nnfs for 
both counting supported models (\*.sm.\*) and answer sets (\*.as.\*)

* obtain a CCG from the answer-sets-encoding nnf with
```
iascar -com -lp example.lp -cnf example.lp.as.cnf -nnf example.lp.as.cnf.nnf > example.as.ccg
```
* obtain a CCG from the supported-models-encoding nnf with
```
iascar -com -lp example.lp -cnf example.lp.sm.cnf -nnf example.lp.sm.cnf.nnf > example.sm.ccg
```
* count answer sets with
```
iascar -ccg -in example.as.ccg
c o a=[]
s SATISFIABLE
c s log10-estimate 0.7781512503836436
c s exact arb int 6
```
* count answer sets under assumptions -9 and 10 with
```
iascar -ccg -in example.as.ccg -a -9 10
c o a=[-9, 10]
s SATISFIABLE
c s log10-estimate 0.3010299956639812
c s exact arb int 2
```
* count answer sets with anytime refinement based one encoded unsupported constraints and with unbounded alternation depth with
```
iascar -car -ccg example.sm.ccg -ucs exmaple.ucs -dep 0
c o d=1 n=1 a=[]  # depth d, number of unsupported constraints n, assumptions a
c o +par          # runs in parallel
c o 0 0.95        # overall log10-count of input ccg
c o 1.00-         # amount of unsupported constraints taken into consideration is 100% (1.00)
                  # and counting stopped on exclusion (-)
s SATISFIABLE
c s log10-estimate 0.7781512503836436
c s exact arb int 6
```
* count answer sets using enumeration
    * uses clingo, hence clingo arguments are permitted, e.g., `--supp-models`
      to count supported models. in particular provide an integer to declare
      the max. number of answer sets to count. `0` stands fo no bound. if
      you provide no integer, iascar will count up to 1 answer set
    * prepend `%` to assumptions where `l` stands for a positive literal and `~l` stands for a negative literal 
```
iascar -enum -in examples/example.lp 0 %a %~f
c ["0"]
c ["a", "~f"]
s SATISFIABLE
c s exact arb int 1
```

* to count on nnfs based on answer set programs use `-nnf -in nnf_path`
* to count on arbitrary nnfs use `-nnfarb -in nnf_path`
