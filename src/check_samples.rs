use anyhow::Result;
use colored::*;
use prettytable::{format, row, table, Table};
use regex::Regex;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::ac_scraper::Samples;
use crate::config::{
    make_compile_command, make_execute_command, ConfigMap, ConfigStrMap, ConfigValue,
    ProblemStrInfo,
};
use crate::util::split_one_line_command;

#[derive(PartialEq, Eq)]
pub enum Status {
    AC,
    WA,
    TLE,
    RE,
    CE,
    WJ,
}

impl Status {
    pub fn as_str(&self) -> &str {
        match self {
            Status::AC => "AC",
            Status::WA => "WA",
            Status::TLE => "TLE",
            Status::RE => "RE",
            Status::CE => "CE",
            Status::WJ => "WJ",
        }
    }
    pub fn as_display_string(&self) -> ColoredString {
        match self {
            Status::AC => "AC".green(),
            Status::WA => "!! WA !!".red(),
            Status::TLE => "!! TLE !!".yellow(),
            Status::RE => "!! RE !!".yellow(),
            Status::CE => "!! CE !!".yellow(),
            Status::WJ => "WJ".into(),
        }
    }

    pub fn from_table_str(v: &str) -> Self {
        match v {
            "AC" => Status::AC,
            "WA" => Status::WA,
            "RE" => Status::RE,
            "TLE" => Status::TLE,
            "CE" => Status::CE,
            "WJ" => Status::WJ,
            _ => {
                let re = Regex::new(r"^(\d+) */ *(\d+) *(.*)$").unwrap();
                if let Some(caps) = re.captures(v) {
                    let status = caps[3].trim();
                    if status == "" {
                        Status::AC
                    } else {
                        Status::from_table_str(status)
                    }
                } else {
                    Status::WJ
                }
            }
        }
    }
}

pub struct FailedDetail {
    pub index: usize,
    pub input: String,
    pub expected: String,
    pub status: Status,
    pub output: String,
}

pub struct SampleResults {
    pub size: usize,
    pub total_status: Status,
    pub failed_details: Vec<FailedDetail>,
}

fn add_total_status_to_table(mut table: Table, total_status: &Status) -> Table {
    let mut total_table = table!([c => "Total", total_status.as_display_string().reverse()]);
    total_table.set_format(*format::consts::FORMAT_BOX_CHARS);
    table.add_row(row![cH2 => total_table]);
    table
}

fn compile(problem_str_info: &ProblemStrInfo, config_str_map: &ConfigStrMap) -> Result<Status> {
    let mut compile_config: HashMap<String, String> = HashMap::new();
    compile_config.extend(
        config_str_map
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone())),
    );
    compile_config.extend(
        problem_str_info
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone())),
    );

    let compile_command = make_compile_command(compile_config)?;

    let (command, args) = split_one_line_command(&compile_command);
    println!("{}", format!("{:-^30}", " Compile ").blue());
    println!("{}: {}", "Compile command".green(), command);
    println!("{}: {:?}", "Compile arguments".green(), args);
    println!("{}", "Compiling...".green());
    let compile_status = Command::new(command)
        .args(&args)
        .status()
        .expect("Failed to execute compilation");

    let status = if compile_status.success() {
        println!("{}", "Compiled successfully.".green());
        Status::AC
    } else {
        println!("{}", "Compilation has failed!".yellow());
        Status::CE
    };

    Ok(status)
}

pub fn execute_with_manual_input(
    problem_str_info: &ProblemStrInfo,
    config_str_map: &ConfigStrMap,
) -> Result<()> {
    println!("{}", format!("{:-^30}", " Manual input mode ").blue());
    let mut execute_config: HashMap<String, String> = HashMap::new();
    execute_config.extend(
        config_str_map
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone())),
    );
    execute_config.extend(
        problem_str_info
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone())),
    );
    let execute_command = make_execute_command(execute_config)?;
    let (command, args) = split_one_line_command(&execute_command);

    println!("{}: {}", "Execute command".green(), command);
    println!("{}: {:?}", "Execute arguments".green(), args);

    println!("{}", format!("{:-^30}", " Your input ").blue());
    let child = Command::new(command)
        .args(&args)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run");

    let raw_output = child.wait_with_output().expect("Failed to read stdout!");
    let output_str = String::from_utf8_lossy(&raw_output.stdout).to_string();
    println!("{}", format!("{:-^30}", " Output ").blue());
    println!("{}", output_str);

    Ok(())
}

fn is_ac(expected: &str, output: &str) -> bool {
    let expected_vec: Vec<&str> = expected
        .split([' ', '\n'])
        .filter(|s| !s.is_empty())
        .collect();
    let output_vec: Vec<&str> = output
        .split([' ', '\n'])
        .filter(|s| !s.is_empty())
        .collect();
    expected_vec == output_vec
}

pub fn sample_check(
    problem_str_info: &ProblemStrInfo,
    samples: &Samples,
    config_str_map: &ConfigStrMap,
    config_map: &ConfigMap,
) -> Result<SampleResults> {
    let mut table = table!([c => "Sample", "Status"]);
    let mut failed_details: Vec<FailedDetail> = vec![];
    table.set_format(*format::consts::FORMAT_BOX_CHARS);

    let mut total_status = if let ConfigValue::Boolean(true) = config_map
        .get("need_to_compile")
        .unwrap_or(&ConfigValue::Boolean(false))
    {
        compile(&problem_str_info, &config_str_map)?
    } else {
        Status::AC
    };

    println!("{}", format!("{:-^30}", " Check ").blue());
    if total_status != Status::AC {
        table = add_total_status_to_table(table, &total_status);
        table.printstd();
        return Ok(SampleResults {
            size: 0,
            total_status,
            failed_details: vec![],
        });
    }

    let mut execute_config: HashMap<String, String> = HashMap::new();
    execute_config.extend(
        config_str_map
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone())),
    );
    execute_config.extend(
        problem_str_info
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone())),
    );
    let execute_command = make_execute_command(execute_config)?;
    let (command, args) = split_one_line_command(&execute_command);

    println!("{}: {}", "Execute command".green(), command);
    println!("{}: {:?}", "Execute arguments".green(), args);

    for i in 0..samples.size {
        let mut child = Command::new(command)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to run!");
        let mut stdin = child.stdin.take().expect("Failed to open stdin!");
        let input: String = samples.inputs[i].clone();
        std::thread::spawn(move || {
            stdin
                .write_all(input.as_bytes())
                .expect("Failed to write to stdin!");
        });
        let raw_output = child.wait_with_output().expect("Failed to read stdout!");
        let output_str = String::from_utf8_lossy(&raw_output.stdout).to_string();

        let expected: String = samples.outputs[i].clone();

        let status = if is_ac(&expected, &output_str) {
            Status::AC
        } else {
            total_status = Status::WA;
            failed_details.push(FailedDetail {
                index: i,
                input: samples.inputs[i].clone(),
                status: Status::WA,
                expected,
                output: output_str,
            });
            Status::WA
        };
        table.add_row(row![c => (i+1), status.as_display_string()]);
    }

    table = add_total_status_to_table(table, &total_status);
    table.printstd();

    Ok(SampleResults {
        size: samples.size,
        total_status,
        failed_details,
    })
}

pub fn display_failed_detail(failed_details: Vec<FailedDetail>) {
    let mut table = table!(["Index", "Input", "Expected".green(), "Output".red()]);
    table.set_format(*format::consts::FORMAT_BOX_CHARS);
    for detail in failed_details {
        table.add_row(row![
            detail.index + 1,
            detail.input,
            detail.expected,
            detail.output
        ]);
    }
    table.printstd();
}
