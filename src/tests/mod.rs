use crate::value::Value;
use crate::{parse, VM};

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
        false
    }
}

#[cfg(test)]
mod testcases {
    use crate::value::Value;

    #[test]
    fn test_all() {
        let testsuite = [(include_str!("tests/arithmetic.glc"), Value::Int(682))];

        for (content, expected) in testsuite.iter() {
            assert!(super::test_file(content.to_string(), expected.clone()));
        }
    }
}
