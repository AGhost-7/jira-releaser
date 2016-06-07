extern crate regex;

use regex::Regex;

pub struct ComplexPattern {
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
        let spl_str = String::from("") + "[ ]+" + tag_seperator + "[ ]+";
        let spl_reg = Regex::new(&spl_str).unwrap();

        ComplexPattern {
            project_id: project_id.to_uppercase(),
            predicate_str: pred_str,
            splitter_str: spl_str,
            predicate: pred_reg,
            splitter: spl_reg
        }
    }

    fn find(&self, log: &str) -> Option<ComplexMatch> {
        println!("predicate: {}", self.predicate);
        match self.predicate.captures(log) {
            None => None,
            Some(cap) => {
                println!("cap: {:?}", cap);
                let inner = cap.name("inner").unwrap();
                let mut identifiers: Vec<String> = Vec::new();
                for m in self.splitter.split(inner) {
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

#[test]
#[ignore]
fn predicate_pattern() {
    let pred_str = ComplexPattern::predicate("foo", "-", "\\[", "\\]", "[ ]+&[ ]+");
    let pred = Regex::new(&pred_str).unwrap();
    assert!(pred.is_match("[foo-1 & foo-2] hello world!"));
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
    let found = pat.find("[foo-20]").unwrap();
    assert_eq!(found, ["FOO-20"]);
    assert!(pat.find("[foo ]").is_none(), "is none");
    assert_eq!(pat.find("[foo-9 & foo10]").unwrap(), ["FOO-9", "FOO-10"]);
    let pat2 = ComplexPattern::new("hello", "-", "[(]", "[)]", "[ ]+");
    println!("() pattern:");
    println!("result: {:?}",pat2.find("(hello-10 hello-2) YOLO").unwrap());
    assert_eq!(pat2.find("(hello-10 hello-2) YOLO").unwrap(), ["HELLO-10", "HELLO-2"]);

}
