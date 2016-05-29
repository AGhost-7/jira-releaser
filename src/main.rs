extern crate regex;

use std::process::Command;
use regex::Regex;

mod parameters;
use parameters::Params;

// Returns the git log diff or the latest and release branches.
fn git_logs(params: &Params) -> Result<String, String> {
    let mut cmd = Command::new("git");
    let release_ptr = String::from("^") + (&params.release_branch);
    cmd
        .arg("log")
        .arg(&params.latest_branch)
        .arg(&release_ptr)
        .arg("--no-merges")
        .arg("--pretty=%s");

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .map_err(|_| "Could not parse git command stdout".to_owned())
            } else {
                match String::from_utf8(output.stderr) {
                    Ok(e) => {
                        Err("Error running git command".to_owned() + (&e))
                    },
                    Err(_) => {
                        Err("Error running git command: cannot parse stderr".to_owned())
                    }
                }
            }
        },
        Err(_) => {
            Err("Git command returned an error".to_owned())
        }
    }
}

fn parse_jira_identifiers(logs: String) -> Vec<String> {
    let mut parsed: Vec<String> = Vec::new();
    let snip_reg = Regex::new("^\\[?(?P<tag>[:alpha:]+)(-| )(?P<num>[0-9]+)\\]?").unwrap();
    for line in logs.lines() {
        if let Some(capture) = snip_reg.captures(line) {
            let tag = capture.name("tag").unwrap();
            let num = capture.name("num").unwrap();
            let iden = tag.to_uppercase() + "-" + num;
            parsed.push(iden);
        }
    }
    parsed
}

#[test]
fn test_jira_parser() {
    let mock_logs = "[FOO-123] hello world!\n\
        this wont show up\n\
        bam-12 Another one\n\
        [bam 20] valid\n\
        Saw3 2 heh";
    let parsed = parse_jira_identifiers(String::from(mock_logs));
    assert!(parsed.contains(&String::from("FOO-123")));
    assert!(parsed.contains(&String::from("BAM-12")));
    assert!(parsed.contains(&String::from("BAM-20")));
    assert_eq!(parsed.len(), 3);
}

fn main() {
    let parser = parameters::ParamsParser::new();
    let params: Params = parser.parse_params();
    println!("{:?}", params);
    let logs = git_logs(&params);
    match logs {
        Ok(_) => {
        },
        Err(e) => println!("{}", e)
    };
}
