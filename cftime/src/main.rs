mod datetimes;
// use cftime::cf_parser;
use datetimes::*;
// fn parse(input: &str) {
//     println!("{:?}", cf_parser(input).unwrap())
// }

fn main() {
    let dt = DateTimeProlepticGregorian::from_ymd(2022, 07, 25);
    println!("{}", DateProlepticGregorian::from_timestamp(86399))
}
