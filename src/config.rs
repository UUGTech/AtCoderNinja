use anyhow::{anyhow, Context, Result};
use std::{collections::HashMap, env, fs, io::Write, path::PathBuf};

use crate::util::*;

use serde::{Deserialize, Serialize};
use shellexpand::full;
use std::path::Path;

const CONFIG_DIR: &str = "~/.config/ac-ninja";
const CONFIG_PATH: &str = "~/.config/ac-ninja/config.toml";
const DEFAULT_CONFIG: &str = "#config.toml
work_space = \"~/CompetitiveProgramming\"
need_to_compile = true
output_file_path = \"{{work_space}}/{{CONTEST_TYPE}}/{{contest_id_0_pad}}/a.out\"
source_file_path = \"{{work_space}}/{{CONTEST_TYPE}}/{{contest_id_0_pad}}/{{problem_id}}.cpp\"
compile_command = \"g++ {{source_file_path}} -std=c++17 -o {{output_file_path}}\"
execute_command = \"{{output_file_path}}\"

";

#[derive(Debug, Clone, Serialize)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Vector(ConfigVector),
    Map(ConfigMap),
}
pub type ConfigMap = HashMap<String, ConfigValue>;
pub type ConfigStrMap = HashMap<String, String>;
pub type ConfigVector = Vec<ConfigValue>;

impl ConfigValue {
    pub fn to_string(&self) -> String {
        match self {
            ConfigValue::String(s) => s.clone(),
            ConfigValue::Integer(i) => i.to_string(),
            ConfigValue::Float(f) => f.to_string(),
            ConfigValue::Boolean(b) => b.to_string(),
            ConfigValue::Vector(arr) => {
                let mut buf = String::new();
                buf.push('[');
                for v in arr {
                    buf.push_str(v.to_string().as_str());
                    buf.push_str(", ");
                }
                if arr.len() > 0 {
                    buf.pop();
                    buf.pop();
                }
                buf.push(']');
                buf
            }
            ConfigValue::Map(mp) => {
                let mut buf = String::new();
                for (k, v) in mp {
                    buf.push('\"');
                    buf.push_str(k);
                    buf.push_str(" = ");
                    buf.push_str(v.to_string().as_str());
                }
                buf
            }
        }
    }
}

trait PushConfigValue {
    fn push_string(&mut self, value: String);
    fn push_integer(&mut self, value: i64);
    fn push_float(&mut self, value: f64);
    fn push_boolean(&mut self, value: bool);
    fn push_vector(&mut self, value: ConfigVector);
}

impl PushConfigValue for ConfigVector {
    fn push_string(&mut self, value: String) {
        self.push(ConfigValue::String(full(&value).unwrap().to_string()));
    }
    fn push_integer(&mut self, value: i64) {
        self.push(ConfigValue::Integer(value));
    }
    fn push_float(&mut self, value: f64) {
        self.push(ConfigValue::Float(value));
    }
    fn push_boolean(&mut self, value: bool) {
        self.push(ConfigValue::Boolean(value));
    }
    fn push_vector(&mut self, value: ConfigVector) {
        self.push(ConfigValue::Vector(value));
    }
}

trait InsertConfigValue {
    fn insert_string(&mut self, key: String, value: String);
    fn insert_integer(&mut self, key: String, value: i64);
    fn insert_float(&mut self, key: String, value: f64);
    fn insert_boolean(&mut self, key: String, value: bool);
    fn insert_vector(&mut self, key: String, value: ConfigVector);
    fn insert_map(&mut self, key: String, value: ConfigMap);
}

impl InsertConfigValue for ConfigMap {
    fn insert_string(&mut self, key: String, value: String) {
        self.insert(key, ConfigValue::String(full(&value).unwrap().to_string()));
    }
    fn insert_integer(&mut self, key: String, value: i64) {
        self.insert(key, ConfigValue::Integer(value));
    }
    fn insert_float(&mut self, key: String, value: f64) {
        self.insert(key, ConfigValue::Float(value));
    }
    fn insert_boolean(&mut self, key: String, value: bool) {
        self.insert(key, ConfigValue::Boolean(value));
    }
    fn insert_vector(&mut self, key: String, value: ConfigVector) {
        self.insert(key, ConfigValue::Vector(value));
    }
    fn insert_map(&mut self, key: String, value: ConfigMap) {
        self.insert(key, ConfigValue::Map(value));
    }
}

pub trait ToHashMapString {
    fn to_hash_map_string(&self) -> HashMap<String, String>;
}

impl ToHashMapString for ConfigMap {
    fn to_hash_map_string(&self) -> HashMap<String, String> {
        let mut buf: HashMap<String, String> = HashMap::new();
        for (k, v) in self {
            buf.insert(k.clone(), v.to_string());
        }
        buf
    }
}

fn toml_into_config_map(table: toml::Table, mut config_map: ConfigMap) -> ConfigMap {
    for (key, value) in table {
        match value {
            toml::Value::String(s) => config_map.insert_string(key, s),
            toml::Value::Integer(i) => config_map.insert_integer(key, i),
            toml::Value::Boolean(b) => config_map.insert_boolean(key, b),
            toml::Value::Float(f) => config_map.insert_float(key, f),
            toml::Value::Array(a) => config_map
                .insert_vector(key, toml_into_config_vector(a.clone(), ConfigVector::new())),
            toml::Value::Table(t) => {
                config_map.insert_map(key, toml_into_config_map(t.clone(), ConfigMap::new()))
            }
            _ => (),
        }
    }
    config_map
}

fn toml_into_config_vector(arr: Vec<toml::Value>, mut config_vector: ConfigVector) -> ConfigVector {
    for value in arr {
        match value {
            toml::Value::String(s) => config_vector.push_string(s),
            toml::Value::Integer(i) => config_vector.push_integer(i),
            toml::Value::Boolean(b) => config_vector.push_boolean(b),
            toml::Value::Float(f) => config_vector.push_float(f),
            toml::Value::Array(a) => {
                config_vector.push_vector(toml_into_config_vector(a.clone(), ConfigVector::new()))
            }
            _ => (),
        }
    }
    config_vector
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub work_space: Option<String>,
    pub abc_dir_name: Option<String>,
    pub arc_dir_name: Option<String>,
    pub agc_dir_name: Option<String>,
    pub source_file_path: Option<String>,
    pub execute_command: Option<String>,
    pub compile_command: Option<String>,
    pub need_compile: Option<bool>,
    pub output_file_path: Option<String>,
}

#[derive(Debug)]
pub enum ContestType {
    ABC,
    ARC,
    AGC,
}

impl ContestType {
    pub fn to_string(&self) -> String {
        match self {
            ContestType::ABC => "ABC".to_string(),
            ContestType::ARC => "ARC".to_string(),
            ContestType::AGC => "AGC".to_string(),
        }
    }

    pub fn from_str(v: &str) -> Option<Self> {
        match v {
            "abc" => Some(ContestType::ABC),
            "arc" => Some(ContestType::ARC),
            "agc" => Some(ContestType::AGC),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SourceFilePath {
    work_space: String,
    contest_type: String,
    contest_id: String,
    problem_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OutputFilePath {
    work_space: String,
    contest_type: String,
    contest_id: String,
    problem_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CompileCommand {
    source_file_path: String,
    output_file_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ExecuteCommand {
    source_file_path: String,
    output_file_path: String,
}

#[derive(Debug)]
pub struct ProblemInfo {
    contest_type: ContestType,
    contest_id: i64,
    problem_id: char,
}

pub type ProblemStrInfo = HashMap<String, String>;

// configで定めた通りのファイルの時のみ
pub fn get_problem_info_from_path(
    config_str_map: &HashMap<String, String>,
    problem_id: char,
) -> Result<ProblemInfo> {
    let current_dir = env::current_dir()?.to_str().unwrap().to_string();
    let config_path = str_format(config_str_map["source_file_path"].clone(), config_str_map);
    let source_file_path = PathBuf::from(config_path.clone());
    let source_dir = source_file_path
        .parent()
        .context("Cannot detecet source directory from source_file_path in your config.")?
        .to_str()
        .unwrap()
        .to_string();

    for contest_type in ["abc", "arc", "agc"] {
        for contest_id in 0i64..999i64 {
            let mut mp: HashMap<String, String> = HashMap::new();
            mp.insert("contest_type".to_string(), contest_type.to_string());
            mp.insert("contest_id".to_string(), contest_id.to_string());
            mp.insert(
                "contest_id_0_pad".to_string(),
                format!("{:0>3}", contest_id),
            );
            let cand = str_format(source_dir.clone(), &mp);
            if cand == current_dir {
                let res = ProblemInfo {
                    contest_type: ContestType::from_str(contest_type).unwrap(),
                    contest_id,
                    problem_id,
                };
                return Ok(res);
            }
        }
    }

    let wrong_dir_error: anyhow::Error = anyhow!("If you are not in the directory configured, you need to specify the problem information with options.");
    Err(wrong_dir_error)
}

pub fn get_problem_str_info(problem_info: &ProblemInfo) -> HashMap<String, String> {
    let mut buf: HashMap<String, String> = HashMap::new();
    buf.insert(
        "contest_type".to_string(),
        problem_info.contest_type.to_string().to_lowercase(),
    );
    buf.insert(
        "contest_id".to_string(),
        problem_info.contest_id.to_string(),
    );
    buf.insert(
        "contest_id_0_pad".to_string(),
        format!("{:0>3}", problem_info.contest_id),
    );
    buf.insert(
        "problem_id".to_string(),
        problem_info.problem_id.to_string().to_lowercase(),
    );

    buf
}

pub fn get_config() -> Result<ConfigMap> {
    let path_string = full(CONFIG_DIR)?.to_string();
    let path = Path::new(path_string.as_str());
    if !path.is_dir() {
        fs::create_dir(path)?;
    }

    let path_string = full(CONFIG_PATH)?.to_string();
    let path = Path::new(path_string.as_str());

    if !path.is_file() {
        let mut file = fs::File::create(path)?;
        file.write_all(DEFAULT_CONFIG.as_bytes())?;
    }

    let config_str = fs::read_to_string(path)?;

    let config_toml: toml::Table = toml::from_str(&config_str)?;
    let config_map = toml_into_config_map(config_toml, ConfigMap::new());

    Ok(config_map)
}

pub fn make_compile_command(compile_config: HashMap<String, String>) -> Result<String> {
    let command_format = compile_config
        .get("compile_command")
        .context("Not found compile commnad in your config file.")?
        .to_string();
    let command = str_format(command_format, &compile_config);

    Ok(command)
}

pub fn make_execute_command(execute_config: HashMap<String, String>) -> Result<String> {
    let command_format = execute_config
        .get("execute_command")
        .context("Not found execute commnad in your config file.")?
        .to_string();
    let command = str_format(command_format, &execute_config);

    Ok(command)
}
