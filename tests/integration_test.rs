use pretty_assertions::assert_eq;
use std::process::Command;

use winreg::{
    enums::{RegDisposition, HKEY_CURRENT_USER},
    RegKey,
};

const TEST_KEY: &str = "Software\\rq\\test";

struct TempRegKey {
    name: String,
}

impl TempRegKey {
    fn new() -> Self {
        let name = std::iter::repeat_with(fastrand::alphanumeric)
            .take(16)
            .collect();
        let (key, disp) = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey(format!("{TEST_KEY}\\{name}"))
            .unwrap();

        assert!(
            matches!(disp, RegDisposition::REG_CREATED_NEW_KEY),
            "Test key exists, test not cleaned up?"
        );

        let (def, _) = key.create_subkey("def").unwrap();
        def.set_value("", &"default value").unwrap();

        let (numbers, _) = key.create_subkey("numbers").unwrap();

        let (one, _) = numbers.create_subkey("one").unwrap();
        one.set_value("amount", &1u32).unwrap();
        one.set_value("kind", &"number").unwrap();

        let (two, _) = numbers.create_subkey("two").unwrap();
        two.set_value("amount", &2u32).unwrap();
        two.set_value("kind", &"number").unwrap();

        let (three, _) = numbers.create_subkey("three").unwrap();
        three.set_value("amount", &3u32).unwrap();
        three.set_value("kind", &"number").unwrap();

        eprintln!("Created test key {name}");
        Self { name }
    }

    fn path(&self, path: &str) -> String {
        format!("HKEY_CURRENT_USER\\{TEST_KEY}\\{}\\{path}", self.name)
            .trim_end_matches('\\')
            .to_string()
    }
}

impl Drop for TempRegKey {
    fn drop(&mut self) {
        RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(TEST_KEY)
            .unwrap()
            .delete_subkey_all(&self.name)
            .unwrap();
        eprintln!("Removed test key {}", self.name);
    }
}

fn check(args: &[&str], expected: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_rq"))
        .args(args)
        .output()
        .unwrap();
    let output =
        String::from_utf8(output.stdout).unwrap() + &String::from_utf8(output.stderr).unwrap();
    assert_eq!(output, expected);
}

#[test]
fn test_query() {
    let key = TempRegKey::new();

    let path = key.path("");
    check(
        &[&path],
        &format!(
            r#"{path}\def
{path}\numbers
"#
        ),
    );

    let path = key.path("numbers");
    check(
        &[&path],
        &format!(
            r#"{path}\one
{path}\three
{path}\two
"#
        ),
    );
}

#[test]
fn test_query_recurse() {
    let key = TempRegKey::new();

    let path = key.path("");
    check(
        &[&path, "-s"],
        &format!(
            r#"{path}

{path}\def
    (Default)    REG_SZ    default value

{path}\numbers

{path}\numbers\one
    amount    REG_DWORD    1
    kind    REG_SZ    number

{path}\numbers\three
    amount    REG_DWORD    3
    kind    REG_SZ    number

{path}\numbers\two
    amount    REG_DWORD    2
    kind    REG_SZ    number

"#
        ),
    );
}

#[test]
fn test_query_value() {
    let key = TempRegKey::new();

    let path = key.path("numbers\\one");
    check(
        &[&path, "-s", "-v", "amount"],
        &format!(
            r#"{path}
    amount    REG_DWORD    1

"#
        ),
    );
}

#[test]
fn test_query_value_recurse() {
    let key = TempRegKey::new();

    let path = key.path("");
    check(
        &[&path, "-s", "-v", "amount"],
        &format!(
            r#"{path}

{path}\def

{path}\numbers

{path}\numbers\one
    amount    REG_DWORD    1

{path}\numbers\three
    amount    REG_DWORD    3

{path}\numbers\two
    amount    REG_DWORD    2

"#
        ),
    );
}
