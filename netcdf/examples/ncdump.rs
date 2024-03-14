use clap::Parser;
use netcdf::types::NcVariableType;

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
    let types = g.types()?.collect::<Vec<_>>();
    if !types.is_empty() {
        println!("Types:");
        print_types(&types)?;
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
            println!("): {}", type_name(&v.vartype()));
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

    let types = g.types().collect::<Vec<_>>();
    if !types.is_empty() {
        println!("Types:");
        print_types(&types)?;
    }

    let mut variables = g.variables().peekable();
    if variables.peek().is_some() {
        println!("Variables:");
        for v in variables {
            print!("\t{}", type_name(&v.vartype()));
            print!("(");
            for d in v.dimensions() {
                print!(" {} ", d.name());
            }
            println!("): {}", type_name(&v.vartype()));
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

fn print_types(types: &[NcVariableType]) -> Result<(), Box<dyn std::error::Error>> {
    for t in types {
        match t {
            NcVariableType::Int(_)
            | NcVariableType::String
            | NcVariableType::Float(_)
            | NcVariableType::Char => unreachable!(),
            NcVariableType::Opaque(o) => {
                print!("\t{}: ", o.name);
                println!("Opaque({})", o.size)
            }
            NcVariableType::Enum(e) => {
                print!("\t{}: ", e.name);
                println!("Enum({})", type_name_enum(e))
            }
            NcVariableType::Vlen(v) => {
                print!("\t{}: ", v.name);
                println!("Vlen({})", type_name(&v.basetype))
            }
            NcVariableType::Compound(c) => {
                print!("\t{}: ", c.name);
                print!("Compound({{");
                for field in &c.fields {
                    print!(" {}: {} ", field.name, type_name(&field.basetype));
                }
                println!("}})");
            }
        }
    }
    Ok(())
}
fn type_name(t: &NcVariableType) -> &str {
    use netcdf::types::*;
    match t {
        NcVariableType::Int(IntType::U8) => "u8",
        NcVariableType::Int(IntType::I8) => "i8",
        NcVariableType::Int(IntType::U16) => "u16",
        NcVariableType::Int(IntType::I16) => "i16",
        NcVariableType::Int(IntType::U32) => "u32",
        NcVariableType::Int(IntType::I32) => "i32",
        NcVariableType::Int(IntType::U64) => "u64",
        NcVariableType::Int(IntType::I64) => "i64",
        NcVariableType::Float(FloatType::F32) => "f32",
        NcVariableType::Float(FloatType::F64) => "f64",
        NcVariableType::String => "string",
        NcVariableType::Char => "char",
        NcVariableType::Opaque(x) => &x.name,
        NcVariableType::Enum(x) => &x.name,
        NcVariableType::Vlen(x) => &x.name,
        NcVariableType::Compound(x) => &x.name,
    }
}
fn type_name_enum(t: &netcdf::types::EnumType) -> &str {
    use netcdf::types::EnumTypeValues::*;
    match &t.fieldvalues {
        U8(_) => "u8",
        I8(_) => "i8",
        U16(_) => "u16",
        I16(_) => "i16",
        U32(_) => "u32",
        I32(_) => "i32",
        U64(_) => "u64",
        I64(_) => "i64",
    }
}
