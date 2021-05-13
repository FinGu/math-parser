use mp::math_parser::*;

use std::io::{self, BufRead};

fn main() {
    for line in io::stdin().lock().lines() {
        println!("result: {}", match math_parser::parse(&line.unwrap(), false) {
            Ok(val) => val.to_string(),
            Err(err) => err.to_string()
        });
    }
}
