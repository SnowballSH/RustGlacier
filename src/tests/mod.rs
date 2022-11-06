#[cfg(test)]
mod testcases {
    use crate::{parse, VM};
    use crate::value::Value;

    fn test_file(content: String, expected: Value) -> bool {
        let mut vm = VM {
            repl_mode: true,
            ..Default::default()
        };
        vm.set_source(content.clone());

        let ast_ = parse(content.as_str());
        if let Ok(ast_) = ast_ {
            vm.compile(&ast_);

            if let Some(e) = &vm.error {
                println!("{e}");
                return false;
            }

            vm.optimize();
            vm.execute();

            if let Some(e) = &vm.error {
                println!("{e}");
                return false;
            }

            if let Some(lp) = &vm.last_popped {
                if unsafe { &**lp }.is_equal(&expected) {
                    true
                } else {
                    println!(
                        "Expected: {:?}, got: {:?}",
                        expected.debug_format(),
                        unsafe { &**lp }.debug_format()
                    );
                    false
                }
            } else {
                println!(
                    "Expected: {:?}, got: {:?}",
                    expected.debug_format(),
                    "nothing"
                );
                false
            }
        } else {
            println!("Syntax Error");
            false
        }
    }


    #[test]
    fn test_all() {
        let testsuite = [
            (include_str!("tests/1_arithmetic.glc"), Value::Int(682)),
            (include_str!("tests/2_cf1.glc"), Value::Int(5)),
            (include_str!("tests/3_cf2.glc"), Value::Bool(true)),
        ];

        let mut i = 1;
        for (content, expected) in testsuite.iter() {
            println!("Testing: {i}");
            assert!(test_file(content.to_string(), expected.clone()));
            i += 1;
        }
    }
}
