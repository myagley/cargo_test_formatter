use std::fs::File;
use std::io;

mod junit;
mod parser;

#[derive(Debug, PartialEq)]
pub enum TestResult {
    Ok,
    Ignored,
    Failed,
}
#[derive(Debug, PartialEq)]
pub struct Failure<'a> {
    name: &'a str,
    stdout: &'a str,
    info: &'a str,
    stacktrace: &'a str,
}
#[derive(Debug, PartialEq)]
pub struct Test<'a> {
    name: &'a str,
    result: TestResult,
}
#[derive(Debug, PartialEq)]
pub struct TestModule<'a> {
    result: TestResult,
    tests: Vec<Test<'a>>,
    failures: Vec<Failure<'a>>,
    passed: u32,
    failed: u32,
    ignored: u32,
    measured: u32,
    filtered: u32,
}

fn load_data<'a, T>(reader: &mut T) -> String
where
    T: io::Read + 'a,
{
    let mut string = String::new();

    reader.read_to_string(&mut string).expect("Empty input");

    string
}

fn main() {
    let stdin = io::stdin();
    let string = load_data(&mut stdin.lock());

    let data = parser::parse(&string);

    match data {
        Ok(data) => {
            let output = junit::format(data);
            junit::print(output);
        }
        Err(e) => panic!("Error while parsing:\n{}", e),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_ok() {
        println!("Oh noes!!");
        assert!(true);
    }

    //    #[test]
    //    fn test_failing() {
    //        println!("Oh noes!!");
    //        assert!(false);
    //    }

    #[test]
    #[ignore]
    fn test_failing2() {
        println!("Again!!");
        assert_eq!("no", "yes");
    }
}
