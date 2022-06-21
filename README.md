# ias :car: 
The **Incremental Answer Set Counter with Anytime Refinement**  and **Counting
Graph Compressor** as proposed in [[1]]( https://tinyurl.com/iascar-s).

**iascar** is a propositional model counter for formulae in so called _smooth deterministic decomposable negation normal form_ (sd-DNNF)*[[2]](https://www.tandfonline.com/doi/pdf/10.3166/jancl.11.11-34?casa_token=vUB3KKgEZTEAAAAA:Y_6z-KXBR002dLW60_DjkqjZxo68XCTgLuuBmd3eBPlj98whbWj2pbVAHQTmPTnICCdkimC7gq9J).
In particular, iascar is tailored toward frequent answer set counting of models
under assumptions. However, it can also be used to count
supported models (under assumptions) of a logic program (see Example 2). Even
more so, iascar can also simply be used to count the number of a models of
classic formula. iascar simply expects 
- either an
sd-DNNF in the format as defined in the archive of c2d available from
[http://reasoning.cs.ucla.edu/c2d/](); or 
- a counting graph in [this format]().

## Build 
```console
iascar$ cargo build --release
```
The resulting binary is `target/release/iascar`

## Install 
```console
$ cargo install iascar
```

## Usage
The following describes each use case of iascar demonstrated for answer set
program [`example_lp.lp`](examples/example_lp.lp) and a
[`cnf`](examples/p_xor_q.cnf) of the odd-2-parity function (XOR).

### Assumptions
To provide assumptions use the `-a` flag followed by whitespace seperated
integers, corresponding to literals. Omitting integers or the `-a` flag in
general evaluates to no assumptions.

Literal mappings of an answer set program can usually be found in the original
cnf instance of the program; at least when these
[tools](https://research.ics.aalto.fi/software/asp/download/) are used.
Compressing sd-DNNFs will preserve the original literal mappings and place them
on the beginning of the compressed counting graph (for more see [CCG 
Format]()).
### Example 1 (**Incremental Answer Set Counting with Anytime Refinement**)
To count incrementally with anytime refinement use the `-as` flag and append the
alternation depth. Providing no alternation depth, or providing alternation
depth 0 results in the unbounded alternation depth. 

Note that it is **required**
that you put the unsupported nogood constraints of your instance `instance.lp`
in a file named `instance.cycles` that satisfies the [UNC Format]() and lies on
the same level as `instance.ccg`. 
```console
iascar$ target/release/iascar examples/example_lp.lp -as 1 -a 7 -12
```