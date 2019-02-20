use std::collections::HashMap;
use clap::{ArgMatches};
use std::path::PathBuf;
use std::fs;
use std::io::{BufRead, BufReader};
use std::str;
use std::fs::File;
use crate::csv;
use regex::{Regex, Match};
use std::io::BufWriter;
use std::io::Write;

pub struct InputParam {
    pub name: String,
    pub question: String,
}

pub struct ExpParams {
    pub ion: PathBuf,
    pub fdr: PathBuf,
    pub out: String,
    pub threshold: f32,
    pub ignore: bool
}

#[derive(Debug)]
pub struct Sample {
    condition: String,
    bio_replicate: String,
    run: u32,
}

#[derive(Debug)]
pub struct FDRValue {
    pub value: f32,
    pub blank: bool,
    pub pass: bool,
}

#[derive(Debug)]
pub struct Series {
    pub sample_array: Vec<FDRValue>,
    pub sample_pass: u32,
}

pub fn get_input(args: &ArgMatches, params: &InputParam) -> String {
    if args.is_present(&params.name) {
        return String::from(args.value_of(&params.name).unwrap());
    } else {
        println!("{}", params.question);
        let file_name: String = read!();
        return file_name;
    }
}

pub fn exp_summary(params_map: &HashMap<String, String>) -> ExpParams {

    let e = ExpParams {
        ion: fs::canonicalize(&params_map["ion"]).unwrap(),
        fdr: fs::canonicalize(&params_map["fdr"]).unwrap(),
        out: params_map["out"].clone(),
        threshold: params_map["threshold"].parse().unwrap(),
        ignore: params_map["ignore"].parse().unwrap()
    };
    /*println!("Ion file: {}\nFDR file: {}\nOutput File: {}\nFDR Threshold: {}\nIgnore Blank Rows: {}",
             &params_map["ion"], &params_map["fdr"], &params_map["out"], &params_map["threshold"], &params_map["ignore"]);*/
    println!("Ion file: {:?}\nFDR file: {:?}\nOutput File: {}\nFDR Threshold: {:?}\nIgnore Blank Rows: {:?}",
             &e.ion, &e.fdr, &e.out, &e.threshold, &e.ignore);
    return e
}

pub fn input_generate() -> Vec<InputParam> {
    let params_array = vec![
        InputParam {
            name: String::from("ion"),
            question: String::from("Ion file"),
        },
        InputParam {
            name: String::from("fdr"),
            question: String::from("FDR file"),
        },
        InputParam {
            name: String::from("out"),
            question: String::from("Output file"),
        },
        InputParam {
            name: String::from("threshold"),
            question: String::from("FDR filter threshold"),
        },
        InputParam {
            name: String::from("ignore"),
            question: String::from("Ignore blank rows"),
        }
    ];
    params_array
}

pub fn read_file(file_path: &PathBuf) -> BufReader<File> {
    let file = match fs::File::open(file_path) {
        Err(error) => panic!("Error: {}", error),
        Ok(result) => result,
    };
    return BufReader::new(file);
}

pub fn read_fdr_file(params: &ExpParams) {
    let fdr_file = csv::read_csv(&params.fdr);
    let pattern = Regex::new(r"(.+)_\d+$").unwrap();
    let columns: Vec<&str> = fdr_file.header.split(",").collect();
    let max_col_number = columns.len();
    let mut samples = vec![];
    let mut peptide_map = HashMap::new();
    for column in 9..max_col_number {
        let mut c: &str = &columns.get(column).unwrap();
        c = c.trim_right();
//        let res: Match = pattern.find(c).unwrap();
        let res = pattern.captures(c);
        if let Some(result) = res {
            samples.push(Sample{
                condition: result[1].to_string(),
                bio_replicate: c.to_string(),
                run: (column - 8) as u32,
            });
        }
    }

    for (index, line) in fdr_file.enumerate() {
        let splitted_values: Vec<&str> = line.split(",").collect();
        let mut sample_series: Series = Series {
            sample_array: vec![],
            sample_pass: 0
        };

        for column in 9..max_col_number {
            let mut c: &str = &splitted_values.get(column).unwrap();
            c = c.trim_right();
            if c != "" {
                let fdr_value = match c.parse::<f32>() {
                    Ok(res) => {FDRValue{
                        value: res,
                        blank: false,
                        pass: res < params.threshold,
                    }},
                    Err(_) => {
                        println!("Error parsing value at row {}, column {}", index, column);
                        FDRValue {
                            value: 0.0,
                            blank: true,
                            pass: false,
                        }
                    },
                };
                if !fdr_value.blank && fdr_value.pass {
                    sample_series.sample_pass += 1;
                }
                sample_series.sample_array.push(fdr_value);

            } else {
                println!("Error parsing value at row {}, column {}", index, column);
                sample_series.sample_array.push(FDRValue {
                    value: 0.0,
                    blank: true,
                    pass: false,
                });
            }
            /*            let fdr_value = c.parse::<f32>().unwrap();
                        let k = format!("{},{},{}", splitted_values[0], splitted_values[1], splitted_values[4]);
                        samples_map.get_mut(&column).unwrap().fdr_map.insert(k, FDRValue{ value: fdr_value });*/
        }
        if sample_series.sample_pass > 0 {
            peptide_map.insert(format!("{},{},{}", splitted_values[0], splitted_values[1], splitted_values[4]), sample_series);
        }
    }
    println!("{:?}", &samples);
}

pub fn read_ions_file(params: &ExpParams, fdr_map: HashMap<String, Series>, samples: Vec<Sample>) {
    let ions_file = csv::read_csv(&params.ion);
    let pattern = Regex::new(r"(.+)_\d+$").unwrap();
    let columns: Vec<&str> = ions_file.header.split(",").collect();
    let max_col_number = columns.len();
    let sample_number = max_col_number - 9;
    let out_file = match File::create(&params.out) {
        Ok(fi) => {fi},
        Err(err) => panic!("Error: {}", err),
    };
    let mut writer = BufWriter::new(out_file);
    match write!(writer, "ProteinName,PeptideSequence,PrecursorCharge,FragmentIon,ProductCharge,IsotopeLabelType,Condition,BioReplicate,Run,Intensity") {
        Ok(_) => {},
        Err(err) => println!("Error writing to file: {}", err),
    };
    for line in ions_file {
        let splitted_values: Vec<&str> = line.split(",").collect();
        let k = format!("{},{},{}", splitted_values[0], splitted_values[1], splitted_values[3]);
        if fdr_map.contains_key(&k) {
            let series = fdr_map.get(&k).unwrap();
            if series.sample_pass > 0 {
                for (index, sample) in samples.iter().enumerate() {
                    if series.sample_array[index].pass {
                        match write!(writer,
                               "{},{},{},{},{},{},L,{},{},{}",
                               splitted_values[0],
                               splitted_values[1],
                               splitted_values[3],
                               format!("{}{}", splitted_values[7], splitted_values[8]),
                               splitted_values[6],
                               sample.condition,
                               sample.bio_replicate,
                               sample.run,
                               splitted_values[(sample.run+8) as usize]) {
                            Ok(_) => {},
                            Err(err) => println!("Error writing to file: {}", err),
                        };
                    } else {
                        match write!(writer,
                               "{},{},{},{},{},{},L,{},{},",
                               splitted_values[0],
                               splitted_values[1],
                               splitted_values[3],
                               format!("{}{}", splitted_values[7], splitted_values[8]),
                               splitted_values[6],
                               sample.condition,
                               sample.bio_replicate,
                               sample.run) {
                            Ok(_) => {},
                            Err(err) => println!("Error writing to file: {}", err),
                        };
                    }

                }
            }
        }
    }
    drop(writer);
}