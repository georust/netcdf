#[cfg(feature = "mpi")]
mod feature_gated {
    use mpi::traits::{AsRaw, Communicator};

    fn target_function(rank: i32, t: usize) -> i32 {
        100 * (t as i32) + rank
    }

    fn mpi_null_info() -> mpi_sys::MPI_Info {
        let mut info = std::ptr::null_mut();
        let e = unsafe { mpi_sys::MPI_Info_create(&mut info) };
        assert_eq!(e, mpi_sys::MPI_SUCCESS.try_into().unwrap());

        info
    }

    fn create(
        path: &str,
        communicator: impl Communicator + AsRaw<Raw = mpi_sys::MPI_Comm>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let info = mpi_null_info();
        let mut file =
            netcdf::create_par_with(path, communicator.as_raw(), info, netcdf::Options::empty())?;

        let size = communicator.size() as usize;
        let rank = communicator.rank();

        file.add_dimension("x", size)?;
        file.add_unlimited_dimension("t")?;
        let var = file.add_variable::<i32>("output", &["t", "x"])?;
        var.access_collective()?;

        file.enddef()?;

        let mut var = file.variable_mut("output").unwrap();

        let values = ndarray::Array1::from_shape_fn(10, |t| target_function(rank, t));
        var.put((.., rank as usize), values.view())?;

        Ok(())
    }

    fn read(
        path: &str,
        communicator: impl Communicator + AsRaw<Raw = mpi_sys::MPI_Comm>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let info = mpi_null_info();

        let file =
            netcdf::open_par_with(path, communicator.as_raw(), info, netcdf::Options::empty())?;
        file.access_collective()?;

        let rank = communicator.rank();
        let var = file.variable("output").unwrap();
        let values = var.get::<i32, _>((.., rank as usize))?;

        for (t, &v) in values.iter().enumerate() {
            assert_eq!(v, target_function(rank, t));
        }
        Ok(())
    }

    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        let universe = mpi::initialize().unwrap();
        let path = "par.nc";

        create(path, universe.world())?;

        read(path, universe.world())?;

        Ok(())
    }
}
#[cfg(feature = "mpi")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    feature_gated::main()
}
#[cfg(not(feature = "mpi"))]
fn main() {
    println!("Enable the `mpi` feature to run this example");
}
