#[cfg(feature = "mpi")]
mod parallel;

fn main() {
    #[cfg(feature = "mpi")]
    parallel::main().unwrap();

    #[cfg(not(feature = "mpi"))]
    println!("MPI support is not included, will not run this example");
}
