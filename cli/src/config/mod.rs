use std::fs;
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::DISPLAY;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(rename = "orkaUrl")]
    pub orka_url: String,
}

impl Config {
    /// Initialise the configuration
    ///
    /// Reads file from disk or create it from the default config function
    fn new() -> Self {
        let config_file_location = Config::get_config_path();
        if !config_file_location.exists() {
            Config::generate_default_config()
        }

        let file = match fs::File::open(config_file_location) {
            Ok(file) => file,
            Err(e) => {
                println!("{}", e);
                exit(-1);
            }
        };

        match serde_yaml::from_reader(file) {
            Ok(conf) => {
                let mut final_conf: Config = conf;
                match Config::add_port_to_url(&final_conf.orka_url, 3000) {
                    Ok(new_url) => {
                        final_conf.orka_url = new_url;
                        final_conf
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                        exit(-1)
                    }
                }
            }
            Err(e) => {
                println!("Error parsing configuration file: {}", e);
                exit(-1)
            }
        }
    }

    /// Save the current configuration to disk
    pub fn save(&self) {
        let file_location = Config::get_config_path();
        match serde_yaml::to_string(self) {
            Err(_) => DISPLAY.print_error("Failed to save config !"),
            Ok(config) => {
                match fs::write(file_location, config) {
                    Ok(_) => (),
                    Err(_) => DISPLAY.print_error("Failed to save config !"),
                }
            }
        }
    }

    /// Generate the default configuration
    ///
    /// Create the directory structure and writes the default configuration in the orka config file
    fn generate_default_config() {
        let file_location = Config::get_config_path();
        match fs::create_dir_all(file_location.parent().unwrap()) {
            Ok(_) => (),
            Err(e) => {
                println!("{}", e);
                exit(-1);
            }
        }

        match fs::write(file_location, "orkaUrl: http://localhost\n") {
            Ok(_) => (),
            Err(e) => {
                println!("{}", e);
                exit(-1);
            }
        }
    }

    /// Gather the config path
    ///
    /// Generates it from the user' home
    fn get_config_path() -> PathBuf {
        // FIXME let's hope the home env is defined
        let home = home::home_dir().unwrap();
        home.join(".config").join("orka").join("config.yaml")
    }

    /// Adds our port to the user given url
    ///
    /// Transform the given URL with the given port
    fn add_port_to_url(input_url: &str, port: u16) -> Result<String, url::ParseError> {
        let mut url = Url::parse(input_url)?;
        url.set_port(Some(port)).map_err(|_| url::ParseError::EmptyHost)?;
        Ok(url.into())
    }

    /// Wrap the config struct into an Arc<Mutex>>
    pub fn new_wrapped() -> Arc<Mutex<Config>> {
        let config = Config::new();
        Arc::new(Mutex::new(config))
    }

    /// Change the orka_url parameter to the given input
    pub fn set_orka_url(&mut self, new_url: &str) {
        match Config::add_port_to_url(new_url, 3000) {
            Ok(new_url) => {
                self.orka_url = new_url;
            }
            Err(err) => {
                println!("Error: {}", err);
                exit(-1)
            }
        }
    }
}
