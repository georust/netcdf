use mpi::traits::{AsRaw, Communicator};

fn target_function(rank: i32, t: usize) -> i32 {
    100 * (t as i32) + rank
}

fn mpi_null_info() -> mpi_sys::MPI_Info {
    let mut info = std::mem::MaybeUninit::uninit();
    let e = unsafe { mpi_sys::MPI_Info_create(info.as_mut_ptr()) };
    assert_eq!(e, mpi_sys::MPI_SUCCESS.try_into().unwrap());

    unsafe { info.assume_init() }
}

fn create(
    path: &str,
    communicator: impl Communicator + AsRaw<Raw = mpi_sys::MPI_Comm>,
) -> Result<(), Box<dyn std::error::Error>> {
    let info = mpi_null_info();
    let mut file =
        netcdf::create_par_with(path, communicator.as_raw(), info, netcdf::Options::NETCDF4)?;

    let size = communicator.size() as usize;
    let rank = communicator.rank();

    file.add_dimension("x", size)?;
    file.add_unlimited_dimension("t")?;
    let var = file.add_variable::<i32>("output", &["t", "x"])?;
    var.access_collective()?;

    file.enddef()?;

    let mut var = file.variable_mut("output").unwrap();

    let values = ndarray::Array1::from_shape_fn(10, |t| target_function(rank, t));
    var.put(values.view(), (.., rank as usize))?;

    Ok(())
}

fn read(
    path: &str,
    communicator: impl Communicator + AsRaw<Raw = mpi_sys::MPI_Comm>,
) -> Result<(), Box<dyn std::error::Error>> {
    let info = mpi_null_info();

    let file = netcdf::open_par_with(path, communicator.as_raw(), info, netcdf::Options::empty())?;

    let rank = communicator.rank();
    let var = file.variable("output").unwrap();
    var.access_collective()?;
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
