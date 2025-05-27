use crate::parser;
use rstest::rstest;
use std::fs;
use std::path::PathBuf;

#[rstest]
fn test_suites(#[files("src/tests/**/input.*")] path: PathBuf) {
    let content = fs::read(&path).expect("cannot read the file");
    let result = parser::Parser::default().parse(&content);
    insta::with_settings!({
        omit_expression => true,
        prepend_module_to_snapshot => false,
        snapshot_path => path.parent().unwrap(),
    },{
        insta::assert_debug_snapshot!("result", result);
    });
}
