use std::{
    collections::HashMap,
    fs::{self, create_dir_all, File},
    io::Write,
    path::PathBuf,
    str::FromStr,
};

use anyhow::{anyhow, Context, Result};
use colored::*;
use reqwest::{
    header::{HeaderMap, HeaderValue, COOKIE},
    Response,
};
use scraper::{ElementRef, Html, Selector};
use shellexpand::full;

use crate::{
    config::{ProblemInfo, ProblemStrInfo},
    data::ACN,
    util::str_format,
};

const PARSE_ERROR: &str = "Parse error occurred in getting samples.";
const INPUT_HEADER: &str = "入力例";
const OUTPUT_HEADER: &str = "出力例";
const TASKS_URL: &str = "https://atcoder.jp/contests/{{contest_type}}{{contest_id_0_pad}}/tasks";
const PROBLEM_URL: &str = "https://atcoder.jp/contests/{{contest_type}}{{contest_id_0_pad}}/tasks/{{task_screen_name}}?lang=ja";
const LOGIN_URL: &str = "https://atcoder.jp/login";
const LOCAL_SESSION_PATH: &str = "~/.ac-ninja/session.txt";
const LOCAL_DIR: &str = "~/.ac-ninja";

pub struct Samples {
    pub inputs: Vec<(usize, String)>,
    pub outputs: Vec<(usize, String)>,
    pub size: usize,
}

fn problem_id_to_index(id: &str) -> Result<usize> {
    match id {
        "a" => Ok(0),
        "b" => Ok(1),
        "c" => Ok(2),
        "d" => Ok(3),
        "e" => Ok(4),
        "f" => Ok(5),
        "g" => Ok(6),
        "h" => Ok(7),
        "ex" => Ok(7),
        _ => Err(anyhow!("Failed to convert problem_id to problem index")),
    }
}

pub async fn add_task_name_to_problem_info(
    acn: &ACN,
    mut problem_info: ProblemInfo,
    mut problem_str_info: ProblemStrInfo,
) -> Result<(ProblemInfo, ProblemStrInfo)> {
    let tasks_url = str_format(TASKS_URL.to_string(), &problem_str_info);
    let cookies = load_cookie_headers()?;
    let resp = acn
        .client
        .get(tasks_url.clone())
        .headers(cookies)
        .send()
        .await?
        .error_for_status()?;
    save_cookie(&resp).await?;
    let body = resp.text().await?;
    let doc = Html::parse_document(&body);

    let selctor = Selector::parse("table tbody tr td:nth-child(1)").unwrap();
    let tds = doc.select(&selctor);
    let config_id = problem_str_info.get("problem_id").unwrap();
    for td in tds {
        let id = td
            .first_child()
            .unwrap()
            .first_child()
            .unwrap()
            .value()
            .as_text()
            .unwrap()
            .to_lowercase();
        let href = td
            .first_child()
            .unwrap()
            .value()
            .as_element()
            .unwrap()
            .attr("href")
            .unwrap();
        let now_idx = problem_id_to_index(&id)?;
        let config_idx = problem_id_to_index(config_id)?;
        if now_idx == config_idx {
            let task_screen_name: String = href.split('/').next_back().unwrap().to_string();
            problem_info.task_screen_name = task_screen_name.clone();
            problem_str_info.insert("task_screen_name".to_string(), task_screen_name);
            return Ok((problem_info, problem_str_info));
        }
    }

    Err(anyhow!(
        "Couldn't find {} problem in {}",
        config_id.to_uppercase(),
        tasks_url
    ))
}

fn parse_cookie_string(raw: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut s = raw.trim();
    if let Some(stripped) = s.strip_prefix("Cookie:") {
        s = stripped.trim();
    }
    for part in s.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let mut iter = part.splitn(2, '=');
        if let (Some(k), Some(v)) = (iter.next(), iter.next()) {
            let key = k.trim();
            let val = v.trim();
            if !key.is_empty() {
                map.insert(key.to_string(), val.to_string());
            }
        }
    }
    map
}

fn write_cookie_map(map: &HashMap<String, String>) -> Result<()> {
    if map.is_empty() {
        return Err(anyhow!("Cookie is empty"));
    }
    let local_dir = PathBuf::from_str(&full(&LOCAL_DIR).unwrap())?;
    if !local_dir.is_dir() {
        create_dir_all(local_dir)?;
    }
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    let cookies_str = keys
        .into_iter()
        .map(|k| format!("{}={}", k, map.get(k).unwrap()))
        .collect::<Vec<String>>()
        .join("; ");
    let mut file = File::create(full(&LOCAL_SESSION_PATH).unwrap().to_string())?;
    file.write_all(cookies_str.as_bytes())?;
    Ok(())
}

fn save_cookie_string(cookies_str: &str) -> Result<()> {
    let map = parse_cookie_string(cookies_str);
    write_cookie_map(&map)
}

async fn save_cookie(resp: &Response) -> Result<()> {
    let mut new_map: HashMap<String, String> = HashMap::new();
    for c in resp.cookies() {
        new_map.insert(c.name().to_string(), c.value().to_string());
    }
    if new_map.is_empty() {
        return Ok(());
    }
    let local_path = PathBuf::from_str(&full(&LOCAL_SESSION_PATH).unwrap())?;
    let mut merged = if local_path.is_file() {
        parse_cookie_string(&fs::read_to_string(local_path)?)
    } else {
        HashMap::new()
    };
    for (k, v) in new_map {
        merged.insert(k, v);
    }
    write_cookie_map(&merged)
}

pub async fn ac_logout() -> Result<()> {
    let local_file = PathBuf::from_str(&full(&LOCAL_SESSION_PATH).unwrap())?;
    if local_file.is_file() {
        fs::remove_file(local_file)?;
    }

    Ok(())
}

fn load_cookie_headers() -> Result<HeaderMap> {
    Ok(get_local_session()?.unwrap_or_default())
}

pub async fn ac_login(acn: &ACN) -> Result<()> {
    println!("{}", format!("{:-^30}", " Login ").blue());
    let local_session_path = PathBuf::from_str(&full(&LOCAL_SESSION_PATH).unwrap())?;
    let existing_cookie = if local_session_path.is_file() {
        Some(fs::read_to_string(&local_session_path)?)
    } else {
        None
    };
    let mut existing_map: HashMap<String, String> = HashMap::new();
    if let Some(ref cookie) = existing_cookie {
        for part in cookie.split(';') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let mut iter = part.splitn(2, '=');
            if let (Some(k), Some(v)) = (iter.next(), iter.next()) {
                existing_map.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }
    println!(
        "{}",
        format!("Open {} in a browser and log in.", LOGIN_URL).green()
    );
    println!(
        "{}",
        "Copy REVEL_SESSION (required) and REVEL_FLASH (optional) from DevTools -> Application -> Cookies."
            .green()
    );
    if existing_cookie.is_some() {
        println!(
            "{}",
            "Existing cookie found. Press Enter to keep each value.".green()
        );
    }
    let session = dialoguer::Password::new()
        .with_prompt("REVEL_SESSION")
        .allow_empty_password(existing_map.contains_key("REVEL_SESSION"))
        .interact()?;
    let session = if session.trim().is_empty() {
        existing_map
            .get("REVEL_SESSION")
            .cloned()
            .ok_or_else(|| anyhow!("REVEL_SESSION is required"))?
    } else {
        session
    };
    let flash = dialoguer::Password::new()
        .with_prompt("REVEL_FLASH (optional)")
        .allow_empty_password(true)
        .interact()?;
    let flash = if flash.trim().is_empty() {
        existing_map.get("REVEL_FLASH").cloned().unwrap_or_default()
    } else {
        flash
    };
    let cookie = if flash.trim().is_empty() {
        format!("REVEL_SESSION={}", session)
    } else {
        format!("REVEL_SESSION={}; REVEL_FLASH={}", session, flash)
    };
    save_cookie_string(&cookie)?;
    let _ = acn;
    println!("{}", "Cookie saved. You are now logged in!".magenta());

    Ok(())
}

pub async fn ac_check_login(acn: &ACN) -> Result<bool> {
    let cookies = load_cookie_headers()?;
    if cookies.is_empty() {
        return Ok(false);
    }
    let resp = acn
        .client
        .get("https://atcoder.jp/home")
        .headers(cookies)
        .send()
        .await?
        .error_for_status()?;
    save_cookie(&resp).await?;
    let final_url = resp.url().to_string();
    let body = resp.text().await?;
    if final_url.contains("/login") {
        return Ok(false);
    }
    let doc = Html::parse_document(&body);
    let login_link_selector = Selector::parse("a[href^=\"/login\"]").unwrap();
    let login_link_selector_abs = Selector::parse("a[href^=\"https://atcoder.jp/login\"]").unwrap();
    let logout_link_selector = Selector::parse("a[href^=\"/logout\"]").unwrap();
    let logout_form_selector = Selector::parse("form[action^=\"/logout\"]").unwrap();
    if doc.select(&login_link_selector).next().is_some()
        || doc.select(&login_link_selector_abs).next().is_some()
        || body.contains("Sign In")
        || body.contains("ログイン")
    {
        return Ok(false);
    }
    let has_logout = doc.select(&logout_link_selector).next().is_some()
        || doc.select(&logout_form_selector).next().is_some()
        || body.contains("Sign Out")
        || body.contains("ログアウト")
        || body.contains("/logout");
    Ok(has_logout)
}

pub fn get_local_session() -> Result<Option<HeaderMap>> {
    let local_session_path = PathBuf::from_str(&full(&LOCAL_SESSION_PATH).unwrap())?;
    if !local_session_path.is_file() {
        return Ok(None);
    }
    let contents = fs::read_to_string(local_session_path)?;
    let map = parse_cookie_string(&contents);
    if map.is_empty() {
        return Ok(None);
    }
    let cookies_str = map
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("; ");
    let mut cookie_headers = HeaderMap::new();
    cookie_headers.insert(COOKIE, HeaderValue::from_str(&cookies_str).unwrap());
    Ok(Some(cookie_headers))
}

pub async fn get_sample_cases(
    problem_str_info: &ProblemStrInfo,
    acn: &ACN,
    sample_case_id_arg: Option<usize>,
) -> Result<Samples> {
    let problem_url = str_format(PROBLEM_URL.to_string(), problem_str_info);
    let cookies = load_cookie_headers()?;
    let resp = acn
        .client
        .get(problem_url)
        .headers(cookies)
        .send()
        .await?
        .error_for_status()?;
    save_cookie(&resp).await?;
    let body = resp
        .text()
        .await
        .with_context(|| "Failed to get sample cases. Please check you logged in and try again.")?;
    let doc = Html::parse_document(&body);

    let pre_selector = Selector::parse("pre").unwrap();
    let pre_elements = doc.select(&pre_selector);

    let h3_selector = Selector::parse("h3").unwrap();

    let mut inputs: Vec<(usize, String)> = Vec::new();
    let mut outputs: Vec<(usize, String)> = Vec::new();

    for pre_element in pre_elements {
        let pre_content = pre_element.text().collect::<String>();
        let mut h3_content: Option<String> = None;
        let mut cursor = pre_element.parent();
        while let Some(node) = cursor {
            if let Some(parent) = ElementRef::wrap(node) {
                if let Some(h3_element) = parent.select(&h3_selector).next() {
                    if let Some(text) = h3_element.text().next() {
                        h3_content = Some(text.to_string());
                        break;
                    }
                }
            }
            cursor = node.parent();
        }
        let h3_content = h3_content.context(PARSE_ERROR)?;
        let is_input = h3_content.contains(INPUT_HEADER);
        let is_output = h3_content.contains(OUTPUT_HEADER);
        if is_input {
            let index: usize = h3_content
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap();
            inputs.push((index, pre_content));
        } else if is_output {
            let index: usize = h3_content
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap();
            outputs.push((index, pre_content));
        }
    }
    if let Some(target) = sample_case_id_arg {
        inputs.retain(|x| x.0 == target);
        outputs.retain(|x| x.0 == target);
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
