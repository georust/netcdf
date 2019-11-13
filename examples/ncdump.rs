use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    path: std::path::PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    match run(&opt.path) {
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
        Ok(()) => {
            std::process::exit(0);
        }
    }
}

fn run(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = netcdf::open(path)?;

    println!("{}", file.name());
    print_group(&file)
}

fn print_group(g: &netcdf::group::Group) -> Result<(), Box<dyn std::error::Error>> {
    println!("Group: {}", g.name());
    println!("Dimensions:");
    for d in g.dimensions() {
        if d.is_unlimited() {
            println!("\t{} : Unlimited ({})", d.name(), d.len());
        } else {
            println!("\t{} : ({})", d.name(), d.len());
        }
    }
    println!("types:");
    for t in g.types() {
        print!("\t");
        match t {
            netcdf::types::Type::Opaque(o) => {
                println!("Opaque {} : {} bytes", o.name(), o.size());
            }
            netcdf::types::Type::Enum(e) => {
                println!("Enum {} : {} bytes", e.name(), e.size());
            }
            netcdf::types::Type::Compound(c) => {
                println!("Compound {} : {} bytes", c.name(), c.size());
            }
        }
    }
    println!("Variables:");
    for v in g.variables() {
        print!("\t{}", v.name());
        print!("(");
        for d in v.dimensions() {
            print!(" {} ", d.name());
        }
        println!(")");
        for a in v.attributes() {
            println!("\t\t{} = {:?}", a.name(), a.value()?);
        }
    }
    println!("Attributes:");
    for a in g.attributes() {
        println!("\t\t{} = {:?}", a.name(), a.value()?);
    }
    for g in g.groups() {
        println!();
        print_group(g)?;
    }

    Ok(())
}
