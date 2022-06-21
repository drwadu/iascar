extern crate iascar;

type Result<T> = std::result::Result<T, ExampleError>;

#[derive(Debug)]
enum ExampleError {
    Unknown,
}

const EXAMPLE_LP: &str = "examples/example_lp";

fn main() -> Result<()> {
    let nnf_path = format!("{}.nnf", EXAMPLE_LP);
    iascar::transpiler::transpile(nnf_path).map_err(|_| ExampleError::Unknown)
}
