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
    key: String,

    /// List all registry entries under <key> and its subkeys.
    #[arg(short, long)]
    subkeys: bool,

    /// The name of the registry value to query.
    #[arg(short, long)]
    value: Option<String>,

    /// A glob defining the key name or value to find.
    /// Matches any values containing this value.
    /// Defaults to "*".
    #[arg(short, long)]
    find: Option<String>,

    /// Search only in key names.
    /// Must be used with -f.
    #[arg(short, long)]
    keys: bool,

    /// Search only in key data.
    /// Must be used with -f.
    #[arg(short, long)]
    data: bool,
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

fn print_values(key: &RegKey, filter: Option<&str>) -> io::Result<()> {
    for val in key.enum_values() {
        let (name, val) = val?;
        if filter.is_none_or(|s| s.eq_ignore_ascii_case(&name)) {
            let name = if name.is_empty() { "(Default)" } else { &name };
            println!("    {name}    {:?}    {val}", val.vtype);
        }
    }

    Ok(())
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

    let find_keys = cli.find.is_some() && (cli.keys || !cli.data);

    let items: Vec<_> = match &cli.find {
        Some(f) => par_iter
            .filter(|(_key, path)| !find_keys || path.contains(f))
            .collect(),
        None => par_iter.collect(),
    };

    for (key, path) in items {
        if cli.find.as_ref().is_some_and(|f| !path.contains(f)) {
            continue;
        }
        println!("{path}");
        print_values(&key, cli.value.as_deref()).unwrap();
        println!();
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let (path, root, key) = split_keyname(&cli.key)?;
    let key = RegKey::predef(root).open_subkey(key)?;

    if cli.subkeys {
        walk(key, path, &cli)?;
    } else {
        if key.query_info()?.values > 0 {
            println!("{path}");
            print_values(&key, cli.value.as_deref())?;
            println!();
        }
        for subkey in key.enum_keys() {
            let subkey = subkey?;
            println!("{path}\\{subkey}");
        }
    }

    Ok(())
}
