use std::{
    collections::HashMap,
    fs::{self, create_dir_all, File},
    io::{BufRead, Write},
    path::PathBuf,
    str::FromStr,
};

use anyhow::{anyhow, Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::{
    header::{HeaderMap, HeaderValue, COOKIE},
    Response, StatusCode,
};
use scraper::{ElementRef, Html, Selector};
use shellexpand::full;

use crate::{
    check_samples::Status,
    config::{ConfigMap, ConfigStrMap, ProblemStrInfo},
    data::ACN,
    util::str_format,
};

const PARSE_ERROR: &str = "Parse error occurred in getting samples.";
const INPUT_HEADERS: [&str; 2] = ["Sample Input", "入力例"];
const OUTPUT_HEADERS: [&str; 2] = ["Sample Output", "出力例"];
const CONTEST_URL: &str = "https://atcoder.jp/contests/{{contest_type}}{{contest_id}}/tasks/{{contest_type}}{{contest_id}}_{{problem_id}}";
const SUBMIT_URL: &str = "https://atcoder.jp/contests/{{contest_type}}{{contest_id}}/submit";
const TASK_SCREEN_NAME: &str = "{{contest_type}}{{contest_id}}_{{problem_id}}";
const SUBMISSIONS_URL: &str =
    "https://atcoder.jp/contests/{{contest_type}}{{contest_id}}/submissions/me";
const LOGIN_URL: &str = "https://atcoder.jp/login";
const LOCAL_SESSION_PATH: &str = "~/.ac-ninja/session.txt";
const LOCAL_DIR: &str = "~/.ac-ninja";

pub struct Samples {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub size: usize,
}

async fn get_csrf_token(acn: &ACN, url: &str) -> Result<String> {
    let login_body = acn
        .client
        .get(url)
        .headers(acn.cookies.clone().unwrap_or(HeaderMap::new()))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let login_doc = Html::parse_document(&login_body);
    let selector = Selector::parse("input[name=\"csrf_token\"]").unwrap();
    if let Some(element) = login_doc.select(&selector).next() {
        if let Some(token) = element.value().attr("value") {
            return Ok(token.to_string());
        }
    }
    Err(anyhow!("Failed to get csrf_token"))
}

async fn save_cookie(resp: &Response) -> Result<()> {
    let cookies_str: String = resp
        .cookies()
        .map(|c| format!("{}={}", c.name(), c.value()))
        .collect::<Vec<String>>()
        .join(";");

    let local_dir = PathBuf::from_str(&full(&LOCAL_DIR).unwrap())?;
    if !local_dir.is_dir() {
        create_dir_all(local_dir)?;
    }
    let mut file = File::create(full(&LOCAL_SESSION_PATH).unwrap().to_string())?;
    file.write_all(cookies_str.as_bytes())?;

    Ok(())
}

pub async fn ac_logout() -> Result<()> {
    let local_file = PathBuf::from_str(&full(&LOCAL_SESSION_PATH).unwrap())?;
    if local_file.is_file() {
        fs::remove_file(local_file)?;
    }

    Ok(())
}

pub async fn ac_login(acn: &ACN) -> Result<()> {
    println!("{}", format!("{:-^30}", " Login ").blue());
    let shinobi = "🥷";
    let prompt = "Username".black();
    let username_prompt = format!(" {} {} ", shinobi, prompt).on_white().to_string();
    let username = dialoguer::Input::<String>::new()
        .with_prompt(username_prompt)
        .interact()?;
    let key = "🔒";
    let prompt = "Password".black();
    let password_prompt = format!(" {} {} ", key, prompt).on_white().to_string();
    let password = dialoguer::Password::new()
        .with_prompt(password_prompt)
        .interact()?;

    let csrf_token: String = get_csrf_token(acn, LOGIN_URL).await?;

    let params = [
        ("csrf_token", csrf_token.as_str()),
        ("username", username.as_str()),
        ("password", password.as_str()),
    ];

    let resp = acn
        .client
        .post(LOGIN_URL)
        .headers(acn.cookies.clone().unwrap_or(HeaderMap::new()))
        .form(&params)
        .send()
        .await?;
    save_cookie(&resp).await?;

    let doc = Html::parse_document(&resp.text().await?);

    if let Some(err) = doc
        .select(&Selector::parse("div.alert-danger").unwrap())
        .next()
    {
        let err_msg = err.last_child().unwrap().value().as_text().unwrap().trim();
        if !["You have already signed in.", "すでにログインしています。"].contains(&err_msg)
        {
            ac_logout().await?;
        }
        return Err(anyhow!(
            "Login failed! {}",
            err.last_child().unwrap().value().as_text().unwrap().trim()
        ));
    }

    println!(
        "{}",
        format!("Hello {}, you are now logged in!", username).magenta()
    );

    Ok(())
}

pub fn get_local_session() -> Result<Option<HeaderMap>> {
    let local_session_path = PathBuf::from_str(&full(&LOCAL_SESSION_PATH).unwrap())?;
    if !local_session_path.is_file() {
        return Ok(None);
    }
    let file = std::fs::File::open(local_session_path)?;
    let reader = std::io::BufReader::new(file);

    let mut cookie_headers = HeaderMap::new();
    reader.lines().for_each(|line| {
        cookie_headers.insert(COOKIE, HeaderValue::from_str(&line.unwrap()).unwrap());
    });
    Ok(Some(cookie_headers))
}

struct Submission {
    time: String,
    name: String,
    username: String,
    lang: String,
    score: String,
    id: String,
    status_str: String,
}

fn get_submission_info_from_row(row: &ElementRef) -> Result<Submission> {
    let td_selector = Selector::parse("td").unwrap();
    let mut iter = row.select(&td_selector);
    let time = iter
        .next()
        .unwrap()
        .first_child()
        .unwrap()
        .first_child()
        .unwrap()
        .value()
        .as_text()
        .unwrap()
        .to_string();
    let time = chrono::DateTime::parse_from_str(&time, "%Y-%m-%d %H:%M:%S%z")?;
    let time = time.format("%Y/%m/%d %H:%M:%S").to_string();
    let name = iter
        .next()
        .unwrap()
        .first_child()
        .unwrap()
        .first_child()
        .unwrap()
        .value()
        .as_text()
        .unwrap()
        .to_string();
    let username = iter
        .next()
        .unwrap()
        .first_child()
        .unwrap()
        .first_child()
        .unwrap()
        .value()
        .as_text()
        .unwrap()
        .to_string();
    let lang = iter
        .next()
        .unwrap()
        .first_child()
        .unwrap()
        .first_child()
        .unwrap()
        .value()
        .as_text()
        .unwrap()
        .to_string();
    let score_parent = iter.next().unwrap();
    let id = score_parent.value().attr("data-id").unwrap().to_string();
    let score = score_parent
        .first_child()
        .unwrap()
        .value()
        .as_text()
        .unwrap()
        .to_string();
    iter.next();
    let status_str = iter
        .next()
        .unwrap()
        .first_child()
        .unwrap()
        .first_child()
        .unwrap()
        .value()
        .as_text()
        .unwrap()
        .to_string();
    Ok(Submission {
        time,
        name,
        username,
        lang,
        score,
        id,
        status_str,
    })
}

fn make_submission_display(submission: &Submission) -> String {
    let tate = " | ".blue();
    let score = format!("score: {}", submission.score);
    format!(
        "{}{}{}{}{}{}{}{}{}{}",
        submission.time,
        tate,
        submission.name.green(),
        tate,
        submission.username,
        tate,
        submission.lang,
        tate,
        score,
        tate
    )
}

pub async fn ac_submit(
    acn: &ACN,
    problem_str_info: &ProblemStrInfo,
    config_str_map: &ConfigStrMap,
    config_map: &ConfigMap,
) -> Result<()> {
    println!("{}", format!("{:-^30}", " Submit ").blue());
    let mut data_map: HashMap<String, String> = HashMap::new();
    data_map.extend(config_str_map.iter().map(|(k, v)| (k.clone(), v.clone())));
    data_map.extend(problem_str_info.iter().map(|(k, v)| (k.clone(), v.clone())));

    let submit_file = str_format(config_str_map["source_file_path"].clone(), &data_map);
    println!("{}{}", "Submit file: ".green(), submit_file);
    let source = fs::read(&full(&submit_file).unwrap().to_string())
        .with_context(|| format!("Failed to read {}", submit_file))?;
    let source_str = String::from_utf8_lossy(&source);

    let submit_url = str_format(SUBMIT_URL.to_string(), &data_map);
    let csrf_token: String = get_csrf_token(acn, submit_url.as_str()).await?;

    let task_screen_name = str_format(TASK_SCREEN_NAME.to_string(), &data_map);
    let params = [
        ("data.TaskScreenName", task_screen_name.as_str()),
        (
            "data.LanguageId",
            &config_map.get("language_id").unwrap().to_string(),
        ),
        ("sourceCode", &source_str),
        ("csrf_token", csrf_token.as_str()),
    ];

    println!(
        "{}",
        str_format(
            "Submitting to {{CONTEST_TYPE}}{{CONTEST_ID}} {{PROBLEM_ID}} ...".to_string(),
            &data_map
        )
        .green()
    );
    let resp = acn
        .client
        .post(submit_url.as_str())
        .headers(acn.cookies.clone().unwrap_or(HeaderMap::new()))
        .form(&params)
        .send()
        .await?;

    if resp.status() != StatusCode::OK {
        ac_logout().await?;
        return Err(anyhow!(
            "Submission failed. You may need to login. Try again!"
        ));
    }

    println!("{}", "Submitted".green());

    // check submission result
    let mut submission_result: Status = Status::WJ;
    let mut submission_id: Option<u64> = None;
    let mut all: u64 = 5000;
    let mut done: u64 = 0;
    let bar_init_style = ProgressStyle::with_template("{msg} {bar:80.green/white}")
        .unwrap()
        .progress_chars("##-");
    let bar_progress_style =
        ProgressStyle::with_template("{msg} {bar:80.green/white} {pos:>3}/{len:>3}")
            .unwrap()
            .progress_chars("##-");
    let bar_finish_style = ProgressStyle::with_template("{msg}")
        .unwrap()
        .progress_chars("##-");

    let pb = ProgressBar::new(all)
        .with_message(submission_result.as_display_string().reverse().to_string())
        .with_position(done)
        .with_style(bar_init_style);

    let mut finish = false;
    let mut finish_msg = String::from("");
    let mut timeout_cnt = 0;
    while !finish {
        let submissions_url = str_format(SUBMISSIONS_URL.to_string(), &data_map);
        let req = acn
            .client
            .get(submissions_url)
            .headers(acn.cookies.clone().unwrap_or(HeaderMap::new()))
            .timeout(tokio::time::Duration::from_millis(2000));
        let resp = req.send().await;

        if let Err(e) = resp {
            if e.is_timeout() {
                timeout_cnt += 1;
                if timeout_cnt > 20 {
                    return Err(anyhow!("A lot of timeouts happend. Something went wrong."));
                }
                continue;
            } else {
                return Err(e.into());
            }
        }

        let body = resp.unwrap().text().await;

        if let Err(e) = body {
            if e.is_timeout() {
                timeout_cnt += 1;
                timeout_cnt += 1;
                if timeout_cnt > 20 {
                    return Err(anyhow!("A lot of timeouts happend. Something went wrong."));
                }
                continue;
            } else {
                return Err(e.into());
            }
        }

        let doc = Html::parse_document(&body.unwrap());

        finish_msg = if submission_id.is_none() {
            let tr_selector = Selector::parse("table tbody tr").unwrap();
            let latest_row = doc.select(&tr_selector).next().unwrap();
            let submission = get_submission_info_from_row(&latest_row)?;
            submission_id = Some(submission.id.parse::<u64>().unwrap());
            let status = Status::from_table_str(&submission.status_str);
            if status != Status::WJ {
                pb.set_style(bar_progress_style.clone());
                pb.tick();
            }
            if status.as_str() != submission.status_str {
                let re = Regex::new(r"^(\d+) */ *(\d+) *(.*)$").unwrap();
                if let Some(caps) = re.captures(&submission.status_str) {
                    done = caps.get(1).unwrap().as_str().parse::<u64>().unwrap();
                    all = caps.get(2).unwrap().as_str().parse::<u64>().unwrap();
                    pb.set_length(all);
                    pb.set_position(done);
                }
            } else if status != Status::WJ {
                finish = true;
            }
            let msg = format!(
                "{}  [ {} ]\n",
                make_submission_display(&submission),
                status.as_display_string().reverse()
            );
            submission_result = status;
            pb.set_message(msg.clone());
            msg
        } else {
            let td_selector =
                Selector::parse(format!("td[data-id=\"{}\"]", submission_id.unwrap()).as_str())
                    .unwrap();
            let target_row =
                ElementRef::wrap(doc.select(&td_selector).next().unwrap().parent().unwrap())
                    .unwrap();
            let submission = get_submission_info_from_row(&target_row)?;
            let status = Status::from_table_str(&submission.status_str);
            if status.as_str() != submission.status_str {
                let re = Regex::new(r"^(\d+) */ *(\d+) *(.*)$").unwrap();
                if let Some(caps) = re.captures(&submission.status_str) {
                    done = caps.get(1).unwrap().as_str().parse::<u64>().unwrap();
                    all = caps.get(2).unwrap().as_str().parse::<u64>().unwrap();
                    pb.set_length(all);
                    pb.set_position(done);
                }
            } else if status != Status::WJ {
                finish = true;
            }
            if status != Status::WJ {
                pb.set_style(bar_progress_style.clone());
                pb.tick();
            }
            let msg = format!(
                "{}  [ {} ]\n",
                make_submission_display(&submission),
                status.as_display_string().reverse()
            );
            submission_result = status;
            pb.set_message(msg.clone());
            msg
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    pb.set_style(bar_finish_style.clone());
    pb.tick();
    pb.finish_with_message(finish_msg);

    println!("\nFinished with {}", submission_result.as_display_string());

    Ok(())
}

pub async fn get_sample_cases(
    problem_str_info: &ProblemStrInfo,
    acn: &ACN,
) -> reqwest::Result<Samples> {
    let contest_url = str_format(CONTEST_URL.to_string(), problem_str_info);
    let body = acn
        .client
        .get(contest_url)
        .headers(acn.cookies.clone().unwrap_or(HeaderMap::new()))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let doc = Html::parse_document(&body);

    let pre_selector = Selector::parse("pre").unwrap();
    let pre_elements = doc.select(&pre_selector);

    let h3_selector = Selector::parse("h3").unwrap();

    let mut inputs: Vec<String> = Vec::new();
    let mut outputs: Vec<String> = Vec::new();

    for pre_element in pre_elements {
        let pre_content = pre_element.text().next().expect(PARSE_ERROR);
        let parent_element = pre_element
            .parent()
            .and_then(ElementRef::wrap)
            .expect(PARSE_ERROR);
        let h3_element = parent_element
            .select(&h3_selector)
            .next()
            .expect(PARSE_ERROR);
        let h3_content = h3_element.text().next().expect(PARSE_ERROR);
        let is_input = INPUT_HEADERS.iter().any(|&h| h3_content.contains(h));
        let is_output = OUTPUT_HEADERS.iter().any(|&h| h3_content.contains(h));
        if is_input {
            inputs.push(pre_content.into());
        } else if is_output {
            outputs.push(pre_content.into());
        }
    }

    let size = match inputs.len() == outputs.len() {
        true if !inputs.is_empty() => Some(inputs.len()),
        _ => None,
    }
    .expect(PARSE_ERROR);

    let samples = Samples {
        inputs,
        outputs,
        size,
    };

    Ok(samples)
}
