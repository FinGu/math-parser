use mp::math_parser::*;

use clap::{crate_version, App, Arg};

use std::io::{self, BufRead};

use std::fs;

fn main() -> math_parser_result<()> {
    let matches = App::new("cli-calculator")
        .version(crate_version!())
        .author("FinGu")
        .about("cli for the math parser")
        .arg(
            Arg::with_name("input-string")
                .short("s")
                .long("input-string")
                .help("takes a string and tries to parse it")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("input-file")
                .short("f")
                .long("input-file")
                .help("takes a file and tries to parse its content")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("solves notation step by step"),
        )
        .get_matches();

    let dbg = matches.is_present("debug");

    let parse_pln_panic = |input: Option<String>| -> math_parser_result<()> {
        let (result, dbgres) = match math_parser::parse(&input.expect("Invalid args")) {
            Ok(val) => val,
            Err(err) => return Err(err),
        };

        if dbg {
            dbgres.iter().for_each(|str| println!("{}", str));
        }

        println!("{}", result);

        Ok(())
    };

    if matches.args.is_empty() || (matches.args.len() == 1 && dbg) {
        io::stdin()
            .lock()
            .lines()
            .for_each(|line| match parse_pln_panic(line.ok()) {
                Ok(_) => (),
                Err(x) => println!("{}", x),
            });
    }

    parse_pln_panic(if let Some(input) = matches.value_of("input-string") {
        Some(input.to_owned())
    } else if let Some(file_path) = matches.value_of("input-file") {
        Some(fs::read_to_string(file_path).expect("Invalid file"))
    } else {
        None
    })?;

    Ok(())
}
