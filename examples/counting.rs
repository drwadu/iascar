extern crate iascar;

type Result<T> = std::result::Result<T, ExampleError>;

#[derive(Debug)]
enum ExampleError {
    Unknown,
}

const EXAMPLE_LP: &str = "examples/example_lp";

fn main() -> Result<()> {
    let nnf_path = "examples/p_xor_q.cnf.nnf";
    println!("reading from: {}", &nnf_path);
    let count = iascar::counting::count_on_sddnnf(&nnf_path, &[]);
    println!("counting {:?} models", count);
    let assumptions = &[1, -2];
    let count = iascar::counting::count_on_sddnnf(&nnf_path, assumptions);
    println!(
        "counting {:?} models under assumptions {:?}",
        count, assumptions
    );
    let assumptions = &[-1, 2];
    let count = iascar::counting::count_on_sddnnf(&nnf_path, assumptions);
    println!(
        "counting {:?} models under assumptions {:?}",
        count, assumptions
    );
    println!("--------------------------------------------------------------------------------------------------------------");
    let nnf_path = format!("{}.nnf", EXAMPLE_LP);
    println!("reading from: {}", &nnf_path);
    let assumptions = &[7, -12];
    let count = iascar::counting::count_on_sddnnf_asp(&nnf_path, &[]);
    println!("counting {:?} supported models", count);
    let count = iascar::counting::count_on_sddnnf_asp(nnf_path, assumptions);
    println!(
        "counting {:?} supported models under assumptions {:?}",
        count, assumptions
    );
    println!("--------------------------------------------------------------------------------------------------------------");
    let ccg_path = format!("{}.ccg", EXAMPLE_LP);
    println!("reading from: {}", &ccg_path);
    let cycles = std::fs::read_to_string(format!("{}.cycles", EXAMPLE_LP))
        .map_err(|_| ExampleError::Unknown)?;
    let count = iascar::counting::count_on_cg_with_cycles(
        &ccg_path,
        cycles.lines(),
        &[],
        0, // no bound
    );
    println!(
        "counting {:?} answer sets (unbounded alternation depth: 2)",
        count
    );
    let count = iascar::counting::count_on_cg_with_cycles(
        &ccg_path,
        cycles.lines(),
        &[],
        1, // alternation depth 1
    );
    println!(
        "counting {:?} answer sets (alternation depth: 1) WARNING: potentially undercounting!",
        count
    );
    let count = iascar::counting::count_on_cg_with_cycles(
        &ccg_path,
        cycles.lines(),
        assumptions,
        1, // alternation depth 1
    );
    println!(
        "counting {:?} answer sets under assummptions {:?} (alternation depth: 1) WARNING: potentially undercounting!",
        count, assumptions
    );
    let count = iascar::counting::count_on_cg_with_cycles(
        ccg_path,
        cycles.lines(),
        assumptions,
        0, // no bound
    );
    println!(
        "counting {:?} answer sets under assumptions {:?} (unbounded alternation depth: 2)",
        count, assumptions
    );
    Ok(())
}
