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

pub fn split_one_line_command(one_line: &str) -> (&str, Vec<&str>) {
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

    for caps in re.captures_iter(format_string.as_str()) {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_str_format() {
        let format_string = "I like {{fruit}}.".to_string();
        let mut vals: HashMap<String, String> = HashMap::new();
        vals.insert("fruit".to_string(), "banana".to_string());
        let res = str_format(format_string, &vals);

        assert_eq!("I like banana.", res);

        let format_string = "{{contest_dir}}".to_string();
        let contest_dir = "~/{{contest_type}}/{{contest_id}}".to_string();
        let mut vals: HashMap<String, String> = HashMap::new();
        vals.insert("contest_dir".to_string(), contest_dir);
        vals.insert("contest_type".to_string(), "ABC".to_string());
        vals.insert("contest_id".to_string(), "042".to_string());
        let res = str_format(format_string, &vals);

        assert_eq!("~/ABC/042", res);

        let format_string = "{{contest_dir}}".to_string();
        let contest_dir = "~/{{contest_type}}/{{contest_id}}".to_string();
        let mut vals: HashMap<String, String> = HashMap::new();
        vals.insert("contest_dir".to_string(), contest_dir);
        vals.insert("contest_type".to_string(), "ABC".to_string());
        let res = str_format(format_string, &vals);

        assert_eq!("~/ABC/{{contest_id}}", res);
    }

    #[test]
    fn test_split_one_line_command() {
        let command = "g++ a.cpp -std=c++17 -o a.out";
        let (command, args) = split_one_line_command(command);
        assert_eq!("g++", command);
        assert_eq!(vec!["a.cpp", "-std=c++17", "-o", "a.out"], args);

        let command = "./a.out";
        let (command, args) = split_one_line_command(command);
        assert_eq!("./a.out", command);
        assert!(args.is_empty());
    }
}
