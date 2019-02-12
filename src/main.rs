#[macro_use] extern crate clap;
#[macro_use] extern crate text_io;
extern crate regex;
use std::collections::HashMap;
use clap::{App};
use std::io::{BufRead, BufReader};
use regex::Regex;
use regex::Match;

mod common;
mod csv;


fn main() {

    let mut params_map: HashMap<String, String> = HashMap::new();
    let yaml = load_yaml!("cli.yml");
    let args = App::from_yaml(yaml).get_matches();
    let params_array = common::input_generate();

    for p in params_array {
        params_map.insert(p.name.clone(),  common::get_input(&args, &p));
    }
    let params = common::exp_summary(&params_map);
    let ion_file = csv::read_csv(&params.ion);
    let pattern = Regex::new(r"\d+$").unwrap();
    let columns: Vec<&str> = ion_file.header.split(",").collect();
    for column in 9..columns.len() {
        let mut c: &str = columns.get(column).unwrap();
        c = c.trim_right();
        let res: Match = pattern.find(c).unwrap();
        println!("{}", res.as_str())
    }
    for line in ion_file {
       let buffer: Vec<&str> = line.split(",").collect();
       //println!("{:?}", buffer.len());
    }
}

