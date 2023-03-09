use regex::Regex;
use std::collections::HashMap;

#[macro_export]
macro_rules! cast {
    ($target: expr, $pat: path) => {{
        if let $pat(a) = $target {
            a
        } else {
            panic!("mismatch variant when cast to {}", stringify!($pat));
        }
    }};
}

pub fn split_one_line_command(one_line: &String) -> (&str, Vec<&str>) {
    let mut v: Vec<&str> = one_line.split(' ').collect();
    v.reverse();
    let command = v.pop().unwrap();
    v.reverse();

    (command, v)
}

pub fn str_format(format_string: String, vals: &HashMap<String, String>) -> String {
    let re = Regex::new(r"\{\{.+?\}\}").unwrap();
    let mut keys: Vec<String> = Vec::new();
    let mut labels: Vec<String> = Vec::new();

    for caps in re.captures_iter(&format_string.as_str()) {
        let label = caps[0].to_string().clone();
        let key = &label[2..(label.len() - 2)];
        keys.push(key.to_string());
        labels.push(label);
    }

    let mut res = format_string.clone();
    for (label, key) in labels.iter().zip(keys.iter()) {
        if let Some(value) = vals.get(&key.clone().to_lowercase()) {
            if key.chars().all(|x| !x.is_alphabetic() || x.is_uppercase()) {
                res = res.replace(label, value.to_uppercase().as_str());
            } else {
                res = res.replace(label, value);
            }
        }
    }

    if format_string != res && re.is_match(&res) {
        res = str_format(res.clone(), vals);
    }

    res
}
