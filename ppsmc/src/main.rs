#![feature(get_mut_unchecked)]

mod automata;
mod cav00;
mod command;
mod ltl;
mod property_driven;
mod traditional;
mod util;

use clap::Parser;
use command::Algorithm;
use smv::Smv;

type BddManager = sylvan::Sylvan;
type Bdd = sylvan::Bdd;

fn main() {
    let input_file = "abp8-p0.smv";

    let mut input_file = format!("./benchmark/{}", input_file);
    let args = command::Args::parse();
    if !args.file.is_empty() {
        input_file = args.file.to_string();
    }
    let smv = Smv::from_file(input_file).unwrap();
    let manager = BddManager::init(args.parallel);
    let algorithm = match args.algorithm {
        Algorithm::PropertyDriven => property_driven::check,
        Algorithm::Traditional => traditional::check,
        Algorithm::Cav00 => cav00::check,
    };
    let (res, time) = algorithm(manager, smv, args);
    println!("res: {}, time: {:?}", res, time);
}
