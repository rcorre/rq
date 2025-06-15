use std::io;

use clap::Parser;
use rayon::prelude::*;
use winreg::enums::*;
use winreg::RegKey;
use winreg::HKEY;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Full path of the subkey to query.
    /// The keyname must include a valid root key, being one of: HKLM, HKCU, HKCR, HKU, and HKCC
    root: String,

    /// Return only keys matching this string.
    #[arg(short, long)]
    key: Option<String>,

    /// Return only values matching this string.
    #[arg(short, long)]
    value: Option<String>,

    /// Return only values with data matching this string.
    #[arg(short, long)]
    data: Option<String>,
}

fn split_keyname(keyname: &str) -> io::Result<(String, HKEY, &str)> {
    let (root, subkey) = match keyname.trim_end_matches('\\').split_once('\\') {
        Some((root, key)) => (root, key),
        None => (keyname, ""),
    };

    let (root_name, root_key) = match root.to_uppercase().as_str() {
        "HKEY_LOCAL_MACHINE" | "HKLM" => ("HKEY_LOCAL_MACHINE", HKEY_LOCAL_MACHINE),
        "HKEY_CURRENT_USER" | "HKCU" => ("HKEY_CURRENT_USER", HKEY_CURRENT_USER),
        "HKEY_CLASSES_ROOT" | "HKCR" => ("HKEY_CLASSES_ROOT", HKEY_CLASSES_ROOT),
        "HKEY_USERS" | "HKU" => ("HKEY_USERS", HKEY_USERS),
        "HKEY_CURRENT_CONFIG" | "HKCC" => ("HKEY_CURRENT_CONFIG", HKEY_CURRENT_CONFIG),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid root: '{root}'"),
            ))
        }
    };

    Ok((format!("{root_name}\\{subkey}"), root_key, subkey))
}

fn walk(key: RegKey, path: String, cli: &Cli) -> io::Result<()> {
    let par_iter =
        rayon::iter::walk_tree_prefix((key, path), |(key, path)| -> Vec<(RegKey, String)> {
            key.enum_keys()
                .map(|name| name.unwrap())
                .map(|name| (key.open_subkey(&name).unwrap(), name))
                .map(|(key, name)| (key, format!("{path}\\{name}")))
                .collect::<Vec<_>>()
        });

    let items: Vec<_> = par_iter
        // first filter out keys not matching the filter
        .filter(|(_key, path)| cli.key.as_ref().is_none_or(|key| path.contains(key)))
        // fiter by values
        .filter_map(|(key, path)| {
            let mut values = Vec::new();
            for val in key.enum_values() {
                let (name, value) = val.unwrap();
                if cli.value.as_ref().is_some_and(|v| !name.contains(v)) {
                    continue;
                }
                let data = value.to_string();
                if cli
                    .data
                    .as_ref()
                    .is_some_and(|d| !data.to_string().contains(d))
                {
                    continue;
                }
                values.push((name, data, value.vtype))
            }
            if values.is_empty() && (cli.value.is_some() || cli.data.is_some()) {
                None
            } else {
                Some((values, path))
            }
        })
        .collect();

    for (values, path) in items {
        println!("{path}");
        for (name, val, vtype) in values {
            let name = if name.is_empty() { "(Default)" } else { &name };
            println!("    {name}    {vtype:?}    {val}");
        }
        println!();
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let (path, root, key) = split_keyname(&cli.root)?;
    let key = RegKey::predef(root).open_subkey(key)?;

    walk(key, path, &cli)?;

    Ok(())
}
