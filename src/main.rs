mod ac_scraper;
mod check_samples;
mod config;
mod data;
mod util;

use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use ac_scraper::*;
use anyhow::Result;
use check_samples::*;
use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::*;
use config::*;
use data::*;
use shellexpand::full;
use util::str_format;

#[derive(Debug, Parser)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    subcommand_negates_reqs = true,
    arg_required_else_help = true,
)]
struct Cli {
    #[command(subcommand)]
    subcommand: Option<MiniCommand>,

    #[clap(flatten)]
    args: Option<GlobalArgs>,
}

#[derive(Debug, Args)]
pub struct GlobalArgs {
    /// Only test samples (skip clipboard copy)
    #[arg(short, long)]
    pub local: bool,

    /// Copy to clipboard even if sample results are not AC
    #[arg(short, long)]
    pub force: bool,

    /// Manual input  ** This option doesn't run sample check **
    #[arg(short, long)]
    pub insert: bool,

    /// (Required)
    #[arg(name = "PROBLEM_ID")]
    pub problem_id_arg: ProblemIdArg,

    /// (Optional) Source file [If you specify source_file, ac-ninja will use the given value to override your config.]
    #[arg(name = "SOURCE_FILE")]
    pub source_file: Option<PathBuf>,

    /// If you're not in configured directory as {{contest_dir}}, you need to specify
    /// contest_type and contest_id.
    #[clap(value_enum, long = "type", short = 't', name = "CONTEST_TYPE")]
    pub contest_type_arg: Option<ContestTypeArg>,

    /// [possible values: 1, 2, 3, ... ]
    #[arg(long = "id", short = 'I', name = "CONTEST_ID")]
    pub contest_id_arg: Option<i64>,

    /// [possible values: 1, 2, 3, ... ]
    #[arg(long = "sample", short = 's', name = "SAMPLE_CASE_ID")]
    pub sample_case_id_arg: Option<usize>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ContestTypeArg {
    Abc,
    Arc,
    Agc,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ProblemIdArg {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl ProblemIdArg {
    pub fn as_char(&self) -> char {
        match &self {
            ProblemIdArg::A => 'a',
            ProblemIdArg::B => 'b',
            ProblemIdArg::C => 'c',
            ProblemIdArg::D => 'd',
            ProblemIdArg::E => 'e',
            ProblemIdArg::F => 'f',
            ProblemIdArg::G => 'g',
            ProblemIdArg::H => 'h',
        }
    }
}

impl ContestTypeArg {
    pub fn as_str(&self) -> String {
        match &self {
            ContestTypeArg::Abc => "abc".to_string(),
            ContestTypeArg::Arc => "arc".to_string(),
            ContestTypeArg::Agc => "agc".to_string(),
        }
    }
}

#[derive(Debug, Subcommand)]
enum MiniCommand {
    /// Login to AtCoder, save session to local
    Login,
    /// Logout, delete session file from local
    Logout,
    /// Check if local session cookie is valid
    LoginCheck,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut acn = ACN::new(&cli.args).await?;

    if let Some(subcommand) = cli.subcommand {
        match subcommand {
            MiniCommand::Login => {
                ac_logout().await?;
                ac_login(&acn).await?;
                acn.cookies = get_local_session()?;
                return Ok(());
            }
            MiniCommand::Logout => {
                ac_logout().await?;
                println!("{}", "You are now logged out".green());
                return Ok(());
            }
            MiniCommand::LoginCheck => {
                let ok = ac_check_login(&acn).await?;
                if ok {
                    println!("{}", "Session is valid.".green());
                } else {
                    println!("{}", "Session is invalid. Run `ac-ninja login`.".yellow());
                }
                return Ok(());
            }
        }
    }

    // login check
    if acn.cookies.is_none() {
        println!(
            "{}{}",
            "Local session not found!\n".red(),
            "You need to login at first".green()
        );
        ac_login(&acn).await?;
        acn.cookies = get_local_session()?;
    }

    let cli_args = cli.args.unwrap();

    let (_, problem_str_info) = get_problem_info_from_path(
        &acn,
        &acn.config_str_map,
        cli_args.problem_id_arg.as_char(),
        &cli_args,
    )
    .await?;

    print_problem_info(&problem_str_info)?;

    if cli_args.insert {
        execute_with_manual_input(&problem_str_info, &acn.config_str_map)?;
        return Ok(());
    }
    let samples = get_sample_cases(&problem_str_info, &acn, cli_args.sample_case_id_arg).await?;
    let sample_results = sample_check(
        &problem_str_info,
        &samples,
        &acn.config_str_map,
        &acn.config_map,
    )?;
    if !sample_results.failed_details.is_empty() {
        display_failed_detail(sample_results.failed_details);
    }

    let should_copy = (sample_results.total_status == Status::AC || cli_args.force)
        && !cli_args.local
        && cli_args.sample_case_id_arg.is_none();
    if should_copy {
        if let Err(e) = copy_source_to_clipboard(&problem_str_info, &acn.config_str_map) {
            eprintln!("{} {}", "Failed to copy to clipboard:".red(), e);
        } else {
            println!("{}", "Source copied to clipboard.".green());
        }
    }

    Ok(())
}

fn copy_source_to_clipboard(
    problem_str_info: &ProblemStrInfo,
    config_str_map: &ConfigStrMap,
) -> Result<()> {
    let mut data_map: HashMap<String, String> = HashMap::new();
    data_map.extend(config_str_map.iter().map(|(k, v)| (k.clone(), v.clone())));
    data_map.extend(problem_str_info.iter().map(|(k, v)| (k.clone(), v.clone())));
    let source_file = str_format(config_str_map["source_file_path"].clone(), &data_map);
    let source_path = full(&source_file)?.to_string();
    let source = fs::read(&source_path)?;

    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(&source)?;
    }
    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow::anyhow!("pbcopy failed"));
    }
    Ok(())
}
