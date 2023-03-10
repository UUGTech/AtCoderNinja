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
use clap::{Args, Parser, Subcommand};
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
struct GlobalArgs {
    /// Without submit, only test samples
    #[arg(short, long)]
    local: bool,

    /// Force to submit, even if ther result of the samples is not AC
    #[arg(short, long)]
    force: bool,

    /// Manual input  ** This option don't allow to submit **
    #[arg(short, long)]
    insert: bool,

    /// Problem_id ex: a, b, c ...
    #[arg(name = "PROBLEM_ID")]
    problem_id_arg: char,

    /// [WIP] Source file to be submitted
    #[arg(name = "FILE")]
    source_file: Option<PathBuf>,

    /// [WIP] abc, arc or agc
    #[arg(name = "CONTEST_TYPE")]
    contest_type_arg: Option<String>,

    /// [WIP] contest_id ex:1, 260, ...
    #[arg(name = "CONTEST_ID")]
    contest_id_arg: Option<i64>,
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
    let mut acn = ACN::new().await?;

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

    let problem_info = get_problem_info_from_path(&acn.config_str_map, cli_args.problem_id_arg)?;
    let problem_str_info = get_problem_str_info(&problem_info);

    if cli_args.insert {
        execute_with_manual_input(&problem_str_info, &acn.config_str_map)?;
        return Ok(());
    }
    let samples = get_sample_cases(&problem_str_info).await?;
    let sample_results = sample_check(
        &problem_str_info,
        &samples,
        &acn.config_str_map,
        &acn.config_map,
    )?;
    if !sample_results.failed_details.is_empty() {
        display_failed_detail(sample_results.failed_details);
    }

    if (sample_results.total_status != Status::AC && !cli_args.force) || cli_args.local {
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
