use std::io;
use winreg::enums::*;
use winreg::RegKey;

use clap::Parser;
use winreg::HKEY;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Full path of the subkey to query.
    /// The keyname must include a valid root key, being one of: HKLM, HKCU, HKCR, HKU, and HKCC
    keyname: String,
}

fn split_keyname(keyname: &str) -> io::Result<(HKEY, &str)> {
    let (root, key) = match keyname.split_once('\\') {
        Some((root, key)) => (root, key),
        None => (keyname, ""),
    };

    let root = match root {
        "HKEY_LOCAL_MACHINE" | "HKLM" => HKEY_LOCAL_MACHINE,
        "HKEY_CURRENT_USER" | "HKCU" => HKEY_CURRENT_USER,
        "HKEY_CLASSES_ROOT" | "HKCR" => HKEY_CLASSES_ROOT,
        "HKEY_USERS" | "HKU" => HKEY_USERS,
        "HKEY_CURRENT_CONFIG" | "HKCC" => HKEY_CURRENT_CONFIG,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid root: '{root}'"),
            ))
        }
    };

    Ok((root, key))
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let (root, key) = split_keyname(&cli.keyname)?;
    let key = RegKey::predef(root).open_subkey(key)?;

    for key in key.enum_keys() {
        let key = key?;
        println!("{key}");
    }

    for val in key.enum_values() {
        let (name, val) = val?;
        println!("\t{name}\t{:?}\t{val}", val.vtype);
    }

    Ok(())
}
