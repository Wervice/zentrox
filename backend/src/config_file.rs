use std::fs;
use std::path;
use dirs::home_dir;
use toml_edit::{self, value, DocumentMut};

/// Read from a key-value config file.
///
/// * `key` - Key that leads to the required value
/// The file is stored in the TOML file format.
/// The toml file is located under $HOME/zentrox_data/zentrox_store.toml
/// If the key is not found an empty string "" is returned.
pub fn read(key: &str) -> String {
    let config_file = path::Path::new("")
        .join(dirs::home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("zentrox_store.toml");

    match fs::read_to_string(config_file)
        .expect("Failed to read config file")
        .parse::<toml::Table>()
        .unwrap()
        .get(key)
    {
        Some(value) => value.to_string().replace("\"", ""),
        None => String::from(""),
    }
}

/// Read write to key-value config file.
///
/// * `key` - Key where value is stored
/// * `value` - Value that should be stored
/// The file is stored in the TOML file format.
/// The toml file is located under $HOME/zentrox_data/zentrox_store.toml
/// If the key already exists the current value is overwritten.
/// The function will return (), if everything worked and an error in case of a fail during the
/// write to the config file.
pub fn write(k: &str, v: &str) -> Result<(), std::io::Error> {
    let config_file = path::Path::new("")
        .join(home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("zentrox_store.toml");

    let config_file_swap = path::Path::new("")
        .join(home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("zentrox_store_swap.toml");

    let mut config_file_parsed: DocumentMut = fs::read_to_string(&config_file)
        .expect("Failed to read config file")
        .parse()
        .expect("Failed to parse config file");

    config_file_parsed[k] = value(v);

    let _ = fs::write(&config_file_swap, config_file_parsed.to_string());
    let _ = fs::rename(config_file_swap, config_file);

    Ok(())
}
