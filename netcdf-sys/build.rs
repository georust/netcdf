extern crate gcc;

fn main() {
    // compile c wrapper to convert CPP constants into proper C types+values
    gcc::Config::new()
                .file("src/ncconst.c")
                .include("src")
                .compile("libncconst.a");
}
