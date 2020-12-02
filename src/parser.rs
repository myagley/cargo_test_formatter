//use nom::{alphanumeric, digit, line_ending, not_line_ending, IResult};

use std::str::FromStr;

use nom::{
    alt,
    character::complete::{alphanumeric1, digit1, line_ending, not_line_ending},
    complete, delimited, do_parse, eof, many0, many1, map_res, named, opt, tag, take_until,
    terminated, IResult,
};

use super::{Failure, Test, TestModule, TestResult};

named!(number<&str, u32>,
  map_res!(
    digit1,
    FromStr::from_str
  )
);

/// ok|FAILED|ignored
named!(test_result<&str, TestResult>,
    do_parse!(
        result: alphanumeric1 >>
        ( if result == "ok" {
            TestResult::Ok
        } else if result == "ignored" {
            TestResult::Ignored
        } else {
            TestResult::Failed
        })
    )
);

named!(test_start<&str, u32>, terminated!(
    delimited!(
        tag!("running "),
        number,
        complete!(alt!(
            tag!(" tests") | tag!(" test")
        ))
    ),
    line_ending
));

named!(testsuite_summary<&str, (TestResult,u32, u32, u32, u32, u32)>, do_parse!(
    tag!("test result: ") >>
    result: test_result   >>
    tag!(". ")            >>
    passed: number        >>
    tag!(" passed; ")     >>
    failed: number        >>
    tag!(" failed; ")     >>
    ignored: number       >>
    tag!(" ignored; ")    >>
    measured: number      >>
    tag!(" measured")    >>
    filtered: opt!(delimited!(
        tag!("; "),
        number,
        tag!(" filtered out")
    )) >>
    line_ending                   >>
    (result, passed, failed, ignored, measured, filtered.unwrap_or(0))
));

/// Test line
///  
/// ## Normal test
/// ```
/// test tests::test_test_case ... ok\r\n
/// ```
///
/// # Doc test
/// ```
/// test src/hexfile.rs - hexfile::MBHexFile::new (line 102) ... ok
/// ```
named!(test_function<&str, Test>, do_parse!(
    tag!("test ")       >>
    name: take_until!(" ... ") >> tag!(" ... ") >>
    result: test_result >>
    line_ending >>
    (Test{name, result})
));

named!(failure<&str, Failure>, do_parse!(
    name:   delimited!(tag!("---- "), take_until!(" stdout ----"), tag!(" stdout ----")) >> line_ending >>
    // stdout: opt!(preceded!(tag_s!("\t"), take_until_s!("thread"))) >>
    info:   take_until!("\n\n") >>
    // opt!(terminated!(
    //         tag_s!("note: Run with `RUST_BACKTRACE=1` for a backtrace."), line_ending
    // )) >>
    // stack: opt!(delimited!(
    //         terminated!(tag_s!("stack backtrace:"), line_ending),
    //         take_until_s!("\n\n"),
    //         line_ending
    // )) >>
    line_ending >> line_ending >>
    (Failure{ name, stdout: ""/*stdout.unwrap_or("")*/, info, stacktrace: ""/*stack.unwrap_or("")*/ })
));

named!(failures<&str, Vec<Failure> >, do_parse!(
    terminated!(tag!("failures:"), line_ending) >>
    line_ending >>
    failure_data: many1!(failure) >>
    line_ending >>
    terminated!(tag!("failures:"), line_ending) >>
    many1!(delimited!(tag!("    "), not_line_ending, line_ending)) >>
    line_ending >>
    (failure_data)
));

named!(test_module<&str, TestModule>, do_parse!(
    test_start >>
    tests: terminated!(many0!(test_function), line_ending) >>
    failures: opt!(failures) >>
    end: testsuite_summary >>
    (TestModule{
        result: end.0,
        tests,
        failures: failures.unwrap_or(vec![]),
        passed: end.1,
        failed: end.2,
        ignored: end.3,
        measured: end.4,
        filtered: end.5
    })
));

named!(test_suite<&str, Vec<TestModule> >, terminated!(
    many1!(delimited!(line_ending, test_module,opt!(line_ending))),
    eof!()
));

pub fn parse(string: &str) -> Result<Vec<TestModule>, String> {
    let result: IResult<&str, _> = test_suite(string);
    match result {
        IResult::Ok(("", result)) => Ok(result),
        r => Err(format!("parse failure: {:#?}", r).to_string()),
    }
}

#[cfg(test)]
mod tests {

    use nom::IResult;

    use super::{
        failure, failures, number, testsuite_summary, test_function, test_module, test_result, test_start,
        test_suite,
    };
    use crate::{Failure, Test, TestModule, TestResult};

    #[test]
    fn test_number() {
        assert_eq!(number("0"), IResult::Done("", 0));
        assert_eq!(number("1"), IResult::Done("", 1));
        assert_eq!(number("99999"), IResult::Done("", 99999));
    }

    #[test]
    fn test_test_result() {
        assert_eq!(test_result("ok"), IResult::Done("", TestResult::Ok));
        assert_eq!(test_result("FAILED"), IResult::Done("", TestResult::Failed));
    }

    #[test]
    fn test_test_start() {
        assert_eq!(test_start("running 1 test\r\n"), IResult::Done("", 1));
        assert_eq!(test_start("running 0 tests\r\n"), IResult::Done("", 0));
    }

    #[test]
    fn test_testsuite_summary() {
        assert_eq!(
            testsuite_summary(
                "test result: ok. 60 passed; 2 failed; 3 ignored; 0 measured; 0 filtered out\r\n"
            ),
            IResult::Done("", (TestResult::Ok, 60, 2, 3, 0, 0))
        );
        assert_eq!(
            testsuite_summary(
                "test result: ok. 10 passed; 2 failed; 3 ignored; 4 measured; 0 filtered out\r\n"
            ),
            IResult::Done("", (TestResult::Ok, 10, 2, 3, 4, 0))
        );
        assert_eq!(testsuite_summary("test result: FAILED. 60 passed; 2 failed; 3 ignored; 0 measured; 1 filtered out\r\n"),
      IResult::Done("", (TestResult::Failed,60,2,3,0,1)));
    }

    #[test]
    fn test_test_function() {
        assert_eq!(
            test_function("test tests::test_test_case ... ok\r\n"),
            IResult::Done("", Test("tests::test_test_case", TestResult::Ok))
        );
    }

    #[test]
    fn test_test_failure() {
        assert_eq!(
            failure(include_str!("../tests/test_failure.txt")),
            IResult::Done(
                "",
                Failure(
                    "tests::test_failing2",
                    "Again!!\n",
                    "thread 'tests::test_failing2' panicked at 'assertion failed: \
        `(left == right)` (left: `no`, right: `yes`)', src/main.rs:243",
                    ""
                )
            )
        );
    }

    #[test]
    fn test_test_failures() {
        assert_eq!(
            failures(include_str!("../tests/test_failures.txt")),
            IResult::Done(
                "",
                vec![
                    Failure(
                        "tests::test_failing",
                        "Oh noes!!\n",
                        "thread 'tests::test_failing' panicked at 'assertion failed: \
            false', src/main.rs:250",
                        ""
                    ),
                    Failure(
                        "tests::test_failing2",
                        "Again!!\n",
                        "thread 'tests::test_failing2' panicked at 'assertion failed: \
            `(left == right)` (left: `no`, right: `yes`)', src/main.rs:255",
                        ""
                    )
                ]
            )
        );
    }

    #[test]
    fn test_test_module() {
        assert_eq!(
            test_module(include_str!("../tests/test_module.txt")),
            IResult::Done(
                "",
                TestModule(
                    TestResult::Ok,
                    vec![
                        Test("tests::test_test_case", TestResult::Ok),
                        Test("test_test_case", TestResult::Ok),
                        Test("tests::test_test_CASE::xxx", TestResult::Ok),
                        Test(
                            "src/hexfile.rs - hexfile::MBHexFile::new (line 102)",
                            TestResult::Ok
                        ),
                        Test("tests::test_test_function", TestResult::Ok)
                    ],
                    vec![],
                    1,
                    2,
                    3,
                    4,
                    5
                )
            )
        );

        assert_eq!(test_module(include_str!("../tests/test_module2.txt")),
      IResult::Done("",
          TestModule(
              TestResult::Ok,
              vec![
                  Test("tests::test_test_case",TestResult::Ok),
                  Test("tests::test_test_function",TestResult::Ok)
              ],
              vec![
                  Failure("tests::test_failing",
                      "Oh noes!!\n", "thread \'tests::test_failing\' panicked at \
                      \'assertion failed: false\', src/main.rs:250", ""),
                  Failure("tests::test_failing2",
                      "Again!!\n", "thread \'tests::test_failing2\' panicked at \
                      \'assertion failed: `(left == right)` (left: `no`, right: `yes`)\', src/main.rs:255", "")
              ],1,2,3,4,5)));
    }

    // #[test]
    // fn test_empty_module() {
    //     assert_eq!(test_module(include_str!("../tests/test_empty_module.txt")),
    //   IResult::Done("", TestModule(
    //           TestResult::Ok,vec![], vec![],1,2,3,4,5)));
    // }

    #[test]
    fn test_test_suite() {
        assert_eq!(
            test_suite(include_str!("../tests/test_suite.txt")),
            IResult::Done(
                "",
                vec![
                    TestModule(
                        TestResult::Ok,
                        vec![
                            Test("tests::test_test_case", TestResult::Ok),
                            Test("tests::test_test_function", TestResult::Ok)
                        ],
                        vec![
                            Failure(
                                "tests::test_failing",
                                "Oh noes!!\n",
                                "thread \'tests::test_failing\' panicked at \'assertion failed: \
              false\', src/main.rs:250",
                                ""
                            ),
                            Failure(
                                "tests::test_failing2",
                                "Again!!\n",
                                "thread \'tests::test_failing2\' panicked at \'assertion failed: \
              `(left == right)` (left: `no`, right: `yes`)\', src/main.rs:255",
                                ""
                            )
                        ],
                        1,
                        2,
                        3,
                        4,
                        5
                    ),
                    TestModule(
                        TestResult::Ok,
                        vec![Test(
                            "src/hexfile.rs - hexfile::MBHexFile::new (line 102)",
                            TestResult::Ok
                        ),],
                        vec![],
                        1,
                        0,
                        0,
                        0,
                        0
                    )
                ]
            )
        );
    }
}
