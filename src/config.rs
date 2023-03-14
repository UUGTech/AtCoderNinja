use anyhow::{anyhow, Context, Result};
use std::fmt;
use std::{collections::HashMap, env, fs, io::Write};

use crate::ac_scraper::add_task_name_to_problem_info;
use crate::data::ACN;
use crate::language_id::lang_to_id;
use crate::util::*;

use serde::{Deserialize, Serialize};
use shellexpand::full;
use std::path::Path;

const CONFIG_DIR: &str = "~/.config/ac-ninja";
const CONFIG_PATH: &str = "~/.config/ac-ninja/config.toml";
const DEFAULT_CONFIG: &str =
"#config.toml
#
# 設定に必須な情報はcontest_dir, source_file_path, need_to_compile,
# execute_command, (language_id or language_name)です.
#
# --------------------------------------------------------------------------------------
# contest_dir:          ac-ninjaを実行するディレクトリです.
#                       {{contesty_type}},{{contest_id}}を特定できる必要があります
# --------------------------------------------------------------------------------------
# source_file_path:     ac-ninjaで提出するファイルのパスです.
# --------------------------------------------------------------------------------------
# need_to_compile:      プログラムの実行にコンパイルが必要かどうかを指定します.
#                       trueの場合, {{compile_command}}を指定する必要があります.
# --------------------------------------------------------------------------------------
# execute_command:      プログラムを実行するためのコマンドです.
# --------------------------------------------------------------------------------------
# language_id:          ac-ninjaでの提出に用いる言語のidです.
#                       AtCoderの提出セレクトボックスをディベロッパーツールから見ることで
#                       確認できますが, [早見表](https://github.com/UUGTech/AtCoderNinja/blob/main/LANG_ID.md)が便利です.
# --------------------------------------------------------------------------------------
# language_name:        language_idの代わりに, language_nameを指定することができます.
#                       AtCoderの提出言語セレクトボックスの表示の通りに指定してください.
#                       \"C++(GCC 9.2.1)\", \"Python (3.8.2)\", \"Rust (1.42.0)\"など.
#                       こちらも, [早見表](https://github.com/UUGTech/AtCoderNinja/blob/main/LANG_ID.md)の文字列をコピペすると便利です.
# --------------------------------------------------------------------------------------
# ファイルパスや, 実行コマンドには{{変数}}を含むことができます.
# {{contest_type}}, {{contest_id}}, {{problem_id}}以外の変数は
# config.toml内で解決可能である必要があります.
#
# {{CONTEST_TYPE}}のように大文字で記述すると, \"ABC\",\"ARC\",\"AGC\"のように
# contest_typeが大文字であることを表します.
# また, contest_idに関しては{{contest_id_0_pad}}とすることで, \"009\"のように
# AtCoderのURLに沿った0埋めを表すことが出来ます.
#
# <設定例>
# AtCoderにC++で参戦している人の例です.
# ディレクトリ構成が ~/CompetitiveProgramming/ABC/059/b.cpp のような場合には,
# 以下のような設定が考えられます
# work_space = \"~/CompetitiveProgramming\"
# need_to_compile = true
# contest_dir = \"{{work_space}}/{{CONTEST_TYPE}}/{{contest_id_0_pad}}\"
# output_file_path = \"{{contest_dir}}/a.out\"
# source_file_path = \"{{contest_dir}}/{{problem_id}}.cpp\"
# compile_command = \"g++ {{source_file_path}} -std=c++17 -o {{output_file_path}}\"
# execute_command = \"{{output_file_path}}\"
# language_id = 4003

";

#[derive(Debug, Clone, Serialize, PartialEq)]
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

impl fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigValue::String(s) => write!(f, "{}", s.clone()),
            ConfigValue::Integer(i) => write!(f, "{}", i),
            ConfigValue::Float(v) => write!(f, "{}", v),
            ConfigValue::Boolean(b) => write!(f, "{}", b),
            ConfigValue::Vector(arr) => {
                let mut buf = String::new();
                buf.push('[');
                for v in arr {
                    buf.push_str(v.to_string().as_str());
                    buf.push_str(", ");
                }
                if !arr.is_empty() {
                    buf.pop();
                    buf.pop();
                }
                buf.push(']');
                write!(f, "{}", buf)
            }
            ConfigValue::Map(mp) => {
                let mut buf = String::new();
                for (k, v) in mp {
                    buf.push('\"');
                    buf.push_str(k);
                    buf.push_str(" = ");
                    buf.push_str(v.to_string().as_str());
                }
                write!(f, "{}", buf)
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
#[allow(clippy::upper_case_acronyms)]
pub enum ContestType {
    ABC,
    ARC,
    AGC,
}

impl fmt::Display for ContestType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContestType::ABC => write!(f, "abc"),
            ContestType::ARC => write!(f, "arc"),
            ContestType::AGC => write!(f, "agc"),
        }
    }
}

impl ContestType {
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
    pub contest_type: ContestType,
    pub contest_id: i64,
    pub problem_id: char,
    pub task_screen_name: String,
}

pub type ProblemStrInfo = HashMap<String, String>;

// configで定めた通りのファイルの時のみ
pub async fn get_problem_info_from_path(
    acn: &ACN,
    config_str_map: &HashMap<String, String>,
    problem_id: char,
) -> Result<(ProblemInfo, ProblemStrInfo)> {
    let current_dir = env::current_dir()?.to_str().unwrap().to_string();
    let config_dir = str_format(config_str_map["contest_dir"].clone(), config_str_map);

    for contest_type in ["abc", "arc", "agc"] {
        for contest_id in 0i64..999i64 {
            let mut mp: HashMap<String, String> = HashMap::new();
            mp.insert("contest_type".to_string(), contest_type.to_string());
            mp.insert("contest_id".to_string(), contest_id.to_string());
            mp.insert(
                "contest_id_0_pad".to_string(),
                format!("{:0>3}", contest_id),
            );
            let cand = str_format(config_dir.clone(), &mp);
            if cand == current_dir {
                let problem_info = ProblemInfo {
                    contest_type: ContestType::from_str(contest_type).unwrap(),
                    contest_id,
                    problem_id,
                    task_screen_name: "".into(),
                };
                let problem_str_info = get_problem_str_info(&problem_info);
                let (problem_info, problem_str_info) =
                    add_task_name_to_problem_info(acn, problem_info, problem_str_info).await?;
                return Ok((problem_info, problem_str_info));
            }
        }
    }

    let wrong_dir_error: anyhow::Error = anyhow!("If you are not in the directory configured, you need to specify the problem information with options.");
    Err(wrong_dir_error)
}

fn get_problem_str_info(problem_info: &ProblemInfo) -> HashMap<String, String> {
    let mut buf: HashMap<String, String> = HashMap::new();
    buf.insert(
        "contest_type".to_string(),
        problem_info.contest_type.to_string(),
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
        problem_info.problem_id.to_string(),
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
        return Err(anyhow!(
            "You need to make your configuration at {}",
            full(CONFIG_PATH)?.to_string()
        ));
    }

    let config_str = fs::read_to_string(path)?;

    let config_toml: toml::Table = toml::from_str(&config_str)?;
    let config_map = toml_into_config_map(config_toml, ConfigMap::new());

    let config_map = config_check(config_map)?;

    Ok(config_map)
}

fn config_check(mut config_map: ConfigMap) -> Result<ConfigMap> {
    let need = [
        "need_to_compile",
        "contest_dir",
        "execute_command",
        "source_file_path",
    ];
    let mut miss: Vec<&str> = Vec::new();
    for k in need {
        if !config_map.contains_key(k) {
            miss.push(k);
        }
    }

    if config_map
        .get("need_to_compile")
        .unwrap_or(&ConfigValue::Boolean(false))
        == &ConfigValue::Boolean(true)
        && !config_map.contains_key("compile_command")
    {
        miss.push("compile_command");
    }

    if !config_map.contains_key("language_id") && !config_map.contains_key("language_name") {
        miss.push("language_( id | name )");
    }

    if !miss.is_empty() {
        let miss_str: Vec<String> = miss.iter().map(|&s| s.to_string()).collect();
        return Err(anyhow!(
            "Couldn't find these configurations in your config file: [{}]",
            miss_str.join(", ")
        ));
    }

    let lang_id: i64 = if config_map.contains_key("language_name") {
        let lang_name = &config_map.get("language_name").unwrap().to_string();
        lang_to_id(lang_name)?
    } else {
        let id_in_config = &config_map.get("language_id").unwrap().to_string();
        let id = id_in_config.parse::<i64>();
        if id.is_err() {
            return Err(anyhow!("language_id must be a number"));
        }
        id.unwrap()
    };

    config_map.insert_integer("language_id".to_string(), lang_id);

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
