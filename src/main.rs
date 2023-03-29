mod ac_scraper;
mod check_samples;
mod config;
mod data;
mod language_id;
mod util;

use std::path::PathBuf;

use ac_scraper::*;
use anyhow::Result;
use check_samples::*;
use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::*;
use config::*;
use data::*;

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
    /// Without submit, only test samples
    #[arg(short, long)]
    pub local: bool,

    /// Force to submit, even if ther result of the samples is not AC
    #[arg(short, long)]
    pub force: bool,

    /// Manual input  ** This option don't allow to submit **
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

    if (sample_results.total_status != Status::AC && !cli_args.force)
        || cli_args.local
        || cli_args.sample_case_id_arg.is_some()
    {
        return Ok(());
    }
    ac_submit(
        &acn,
        &problem_str_info,
        &acn.config_str_map,
        &acn.config_map,
    )
    .await?;

    Ok(())
}
