extern crate gcc;

fn main() {
    // compile c wrapper to convert CPP constants into proper C types+values
    gcc::compile_library("libncconst.a", &["src/ncconst.c"]);
}
