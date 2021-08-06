use super::{TestModule, TestResult};

pub fn format(data: Vec<TestModule>) -> xml::Element {
    let mut output = xml::Element::new("testsuites".into(), None, vec![]);

    for module in data {
        let attr = vec![
            ("failures".into(), None, format!("{}", module.failed).into()),
            ("skip".into(), None, format!("{}", module.ignored).into()),
            (
                "tests".into(),
                None,
                format!("{}", module.tests.len()).into(),
            ),
        ];
        let suite = output.tag(xml::Element::new("testsuite".into(), None, attr));

        for test in module.tests {
            let attr = vec![("name".into(), None, test.name.into())];

            let test_xml = suite.tag(xml::Element::new("testcase".into(), None, attr));

            if test.result == TestResult::Ignored {
                test_xml.tag(xml::Element::new("skipped".into(), None, vec![]));
            } else if test.result == TestResult::Failed {
                for failure in &module.failures {
                    if failure.name == test.name {
                        test_xml
                            .tag(xml::Element::new(
                                "failure".into(),
                                None,
                                vec![], //("message".into(), None, failure.info.into())],
                            ))
                            .cdata(failure.info.into());
                        if !failure.stdout.is_empty() {
                            test_xml
                                .tag(xml::Element::new("system-out".into(), None, vec![]))
                                .text(failure.stdout.into());
                        }
                    }
                }
            }
        }
    }

    output
}

pub fn print(output: xml::Element) {
    println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", output);
}
