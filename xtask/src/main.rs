use std::env;
use std::fs::{File, OpenOptions};
use std::io::Write;

fn main() {
    let mut args = env::args().skip(1);
    let test_title = args.next().expect("need test title");

    println!("Creating new test: {test_title}");

    let it_test = OpenOptions::new().append(true).open("tests/it.rs").expect("can't open test/it.rs");

    write!(&it_test, r#"
#[test]
fn {test_title}() {{
    assert_toc!("{test_title}");
}}
"#).unwrap();

    let in_file = File::create(format!("tests/{test_title}.in.md")).unwrap();
    write!(&in_file, r#"# {test_title}

<!-- toc -->

## Title 1

## Title 2
"#).unwrap();

    let out_file = File::create(format!("tests/{test_title}.out.md")).unwrap();
    write!(&out_file, r#"# {test_title}

* [Title 1](#title-1)
* [Title 2](#title-2)

## Title 1

## Title 2
"#).unwrap();
}
