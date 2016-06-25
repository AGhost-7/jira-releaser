extern crate regex;

use regex::Regex;

struct ComplexPattern {
    predicate_str: String,
    splitter_str: String,
    project_id: String,
    predicate: Regex,
    splitter: Regex
}

type ComplexMatch = Vec<String>;

impl ComplexPattern {

    pub fn predicate(project_id: &str,
            tag_seperator: &str,
            open_tag: &str,
            close_tag: &str,
            multi_separator: &str) -> String {

        // Example constructed predicate:
        // "^(?i)[ ]*\\[[ ]*(?P<inner>foo-[0-9]+([ ]+&[ ]+foo-[0-9]+)*)[ ]*\\]"
        // This will find patterns such as this: [foo-1 && foo-2]
        let mut predicate_str = String::from("");
        let items = [
            "^(?i)[ ]*",
            open_tag,
            "[ ]*(?P<inner>",
            project_id,
            tag_seperator,
            "[0-9]+(",
            multi_separator,
            project_id,
            tag_seperator,
            "[0-9]+)*)[ ]*",
            close_tag
        ];

        for item in items.iter() {
            predicate_str.push_str(item);
        }

        predicate_str
    }

    pub fn new(project_id: &str,
            tag_seperator: &str,
            open_tag: &str,
            close_tag: &str,
            multi_separator: &str) -> ComplexPattern {

        // TODO: Sanitize user input: If the project id is a bad regex currently it can
        // cause the program to crash. I want a clean error message instead.
        let pred_str = ComplexPattern::predicate(project_id,
            tag_seperator,
            open_tag,
            close_tag,
            multi_separator);
        let pred_reg = Regex::new(&pred_str).unwrap();
        let spl_str = multi_separator.to_owned();
        let spl_reg = Regex::new(&spl_str).unwrap();

        ComplexPattern {
            project_id: project_id.to_uppercase(),
            predicate_str: pred_str,
            splitter_str: spl_str,
            predicate: pred_reg,
            splitter: spl_reg
        }
    }

    pub fn find(&self, log: &str) -> Option<ComplexMatch> {
        match self.predicate.captures(log) {
            None => None,
            Some(cap) => {
                let inner = cap.name("inner").unwrap();
                let mut identifiers: Vec<String> = Vec::new();
                let split = self.splitter.split(inner);
                for m in split {
                    let mut iden = self.project_id.to_owned() + "-";
                    let mut begin = false;
                    for c in m.chars() {
                        if c.is_digit(10) {
                            begin = true;
                            iden.push(c);
                        } else if begin {
                            break;
                        }
                    }

                    identifiers.push(iden);
                }
                Some(identifiers)
            }
        }
    }
}

pub struct TokenParser {
    patterns: Vec<ComplexPattern>
}

impl TokenParser {
    pub fn new(project_id: &str) -> TokenParser {
        TokenParser {
            patterns: vec![
                // [foo-1 & foo-2] foobar
                ComplexPattern::new(project_id, "-", "\\[", "\\]", "[ ]+&[ ]+"),
                // (foo-1 foo-2) foobar
                ComplexPattern::new(project_id, "-", "[(]", "[)]", "[ ]+"),
                ComplexPattern::new(project_id, "-", "[(]", "[)]", "[ ]*,[ ]+"),
                // foo-1 foo-2 foobar
                ComplexPattern::new(project_id, "-", "", "", "[ ]+"),
                // foo-1, foo-2 foobar
                ComplexPattern::new(project_id, "-", "", "", "[ ]*,[ ]+")
            ]
        }
    }

    pub fn parse(&self, logs: &str) -> Vec<String> {
        let mut jira_tokens: Vec<String> = Vec::new();
        for log in logs.lines() {
            let mut tokens: Vec<String> = Vec::new();
            for pattern in &self.patterns {
                if let Some(found) = pattern.find(log) {
                    if tokens.len() < found.len() {
                        tokens = found;
                    }
                }
            }
            jira_tokens.extend(tokens);
        }
        jira_tokens
    }
}

#[test]
fn predicate_pattern() {
    let pred_str = ComplexPattern::predicate("foo", "-", "\\[", "\\]", "[ ]+&[ ]+");
    let pred = Regex::new(&pred_str).unwrap();
    assert!(pred.is_match("[foo-1 & foo-2] hello world!"));
    assert!(pred.is_match("[foo-20]"));
    assert!(pred.captures("[foo-20]").is_some());
    assert!(pred.is_match(" [ foo-100] foobar"));
    assert_eq!(pred.captures("[ foo-1  ] hai").unwrap().name("inner").unwrap(), "foo-1");
    let pred2_str = ComplexPattern::predicate("foo", "-", "\\(", "\\)", " ");
    let pred2 = Regex::new(&pred2_str).unwrap();
    assert_eq!(pred2.captures("(foo-1 foo-2)").unwrap().name("inner").unwrap(), "foo-1 foo-2");
}

#[test]
fn complex_pattern() {
    // e.g.
    let pat = ComplexPattern::new("foo", "-", "\\[", "\\]", "[ ]+&[ ]+");
    let found = pat.find("[foo-20]");
    assert!(found.is_some(), "found is not some");
    assert_eq!(found.unwrap(), ["FOO-20"]);
    assert!(pat.find("[foo ]").is_none(), "[foo ] isnt none");
    assert_eq!(pat.find("[foo-9 & foo-10]").unwrap(), ["FOO-9", "FOO-10"]);
    let pat2 = ComplexPattern::new("hello", "-", "[(]", "[)]", "[ ]+");
    assert_eq!(pat2.find("(hello-10 hello-2) YOLO").unwrap(), ["HELLO-10", "HELLO-2"]);
}

#[test]
fn token_parser() {
    let parser = TokenParser::new("foo");
    let logs = "[foo-1] hello world
        (foo-2, foo-3) lorem ipsum
        foo-4 tisk tisk
        foo-5 foo-6 yep
        foo-7, foo-8 YERP";

    let jira_tokens = parser.parse(logs);

    let compare: Vec<String> = (1..9)
        .map(|n: u32| String::from("FOO-") + &n.to_string())
        .collect();
    assert_eq!(jira_tokens, compare);
}
