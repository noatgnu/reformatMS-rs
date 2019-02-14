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
    common::read_csv_file(&params);
}

