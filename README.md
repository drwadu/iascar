# ias :car: 
The **Incremental Answer Set Counter with Anytime Refinement**  and **Counting Graph Compressor** as proposed in [[1]]( https://tinyurl.com/iascar-s).

**iascar** is a propositional model counter for formulae in so called _smooth deterministic decomposable negation normal form_ (sd-DNNF) [[2]](https://www.tandfonline.com/doi/pdf/10.3166/jancl.11.11-34?casa_token=vUB3KKgEZTEAAAAA:Y_6z-KXBR002dLW60_DjkqjZxo68XCTgLuuBmd3eBPlj98whbWj2pbVAHQTmPTnICCdkimC7gq9J).
In particular, iascar is tailored toward frequent answer set counting of models under assumptions (see Example 1). However,
it can also be used to count supported models (under assumptions) of a logic program (see Example 2). Even more so, iascar can also simply be used 
to count the number of a models of classic formula (see Example 3), as in the end iascar expects 
- either an sd-DNNF in the format as defined in the archive of c2d available from [http://reasoning.cs.ucla.edu/c2d/](); or 
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
The following describes each use case of iascar using [`example_lp.lp`](examples/example_lp.lp)
### Example 1 (**Incremental Answer Set Counting with Anytime Refinement**)
Let `example.lp` be the following program:
```prolog
a :- not b.
b :- not a.
c :- d, a.
d :- c, a.
e :- not f.
f :- not e.
c :- e.
d :- f.
d :- g.
g :- d.
h :- i.
i :- h.
i :- j.
g :- j.
k :- not j.
j :- not k.
``` 