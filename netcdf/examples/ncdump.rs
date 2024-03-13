use clap::Parser;

#[derive(Debug, Parser)]
struct Opt {
    path: std::path::PathBuf,
}

fn main() {
    let opt = Opt::parse();

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

    println!("{}", file.path()?.to_str().unwrap());
    print_file(&file)
}

fn print_file(g: &netcdf::File) -> Result<(), Box<dyn std::error::Error>> {
    let mut dims = g.dimensions().peekable();
    if dims.peek().is_some() {
        println!("Dimensions:");
        for d in dims {
            if d.is_unlimited() {
                println!("\t{} : Unlimited ({})", d.name(), d.len());
            } else {
                println!("\t{} : ({})", d.name(), d.len());
            }
        }
    }
    let mut types = g.types()?.peekable();
    if types.peek().is_some() {
        println!("Types:");
        for t in types {
            use netcdf::types::VariableType;
            print!("\t{}: ", t.name());
            match t {
                VariableType::Basic(_) | VariableType::String => unreachable!(),
                VariableType::Opaque(o) => println!("Opaque({})", o.size()),
                VariableType::Enum(_) => println!("Enum"),
                VariableType::Vlen(v) => println!("Vlen({})", v.typ().name()),
                VariableType::Compound(c) => {
                    print!("Compound({{");
                    for field in c.fields() {
                        print!(" {}: {} ", field.name(), field.typ().name());
                    }
                    println!("}})");
                }
            }
        }
    }
    let mut variables = g.variables().peekable();
    if variables.peek().is_some() {
        println!("Variables:");
        for v in variables {
            print!("\t{}", v.name());
            print!("(");
            for d in v.dimensions() {
                print!(" {} ", d.name());
            }
            println!("): {}", v.vartype().name());
            for a in v.attributes() {
                println!("\t\t{} = {:?}", a.name(), a.value()?);
            }
        }
    }
    let mut attributes = g.attributes().peekable();
    if attributes.peek().is_some() {
        println!("Attributes:");
        for a in g.attributes() {
            println!("\t\t{} = {:?}", a.name(), a.value()?);
        }
    }
    if let Some(g) = g.root() {
        for g in g.groups() {
            println!();
            print_group(&g)?;
        }
    }

    Ok(())
}

fn print_group(g: &netcdf::Group) -> Result<(), Box<dyn std::error::Error>> {
    println!("Group: {}", g.name());

    let mut dims = g.dimensions().peekable();
    if dims.peek().is_some() {
        println!("Dimensions:");
        for d in dims {
            if d.is_unlimited() {
                println!("\t{} : Unlimited ({})", d.name(), d.len());
            } else {
                println!("\t{} : ({})", d.name(), d.len());
            }
        }
    }
    let mut types = g.types().peekable();
    if types.peek().is_some() {
        println!("Types:");
        for t in types {
            use netcdf::types::VariableType;
            print!("\t{}: ", t.name());
            match t {
                VariableType::Basic(_) | VariableType::String => unreachable!(),
                VariableType::Opaque(o) => println!("Opaque({})", o.size()),
                VariableType::Enum(_) => println!("Enum"),
                VariableType::Vlen(v) => println!("Vlen({})", v.typ().name()),
                VariableType::Compound(c) => {
                    print!("Compound({{");
                    for field in c.fields() {
                        print!(" {}: {} ", field.name(), field.typ().name());
                    }
                    println!("}})");
                }
            }
        }
    }

    let mut variables = g.variables().peekable();
    if variables.peek().is_some() {
        println!("Variables:");
        for v in variables {
            print!("\t{}", v.name());
            print!("(");
            for d in v.dimensions() {
                print!(" {} ", d.name());
            }
            println!("): {}", v.vartype().name());
            for a in v.attributes() {
                println!("\t\t{} = {:?}", a.name(), a.value()?);
            }
        }
    }
    let mut attributes = g.attributes().peekable();
    if attributes.peek().is_some() {
        println!("Attributes:");
        for a in g.attributes() {
            println!("\t\t{} = {:?}", a.name(), a.value()?);
        }
    }
    for g in g.groups() {
        println!();
        print_group(&g)?;
    }

    Ok(())
}
