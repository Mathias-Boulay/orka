use serde::{Serialize, Deserialize, Deserializer};
use std::fs;
use crate::workloads::container::{Container};
use crate::workloads::network::{Network, verify_network};
use thiserror::Error;
use std::io::ErrorKind::NotFound;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Workload kind must be 'container' or 'network'.")]
    UnknownWorkloadKind,
    #[error("File `{0}` not found")]
    FileNotFound(PathBuf),
    #[error("Data could not be read from file")]
    FileCouldNotBeenRead,
    #[error("The ip adress `{0}` is invalid.")]
    InvalidIpAddress(String),
    #[error("The port `{0}` is outside of the allowed port range.")]
    OutsidePortRange(u32),
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum ConfigVariant {
    #[serde(rename(deserialize = "container", serialize = "Container"))]
    Container(Container),

    #[serde(rename(deserialize = "network", serialize = "Network"))]
    Network(Network),
}

#[derive(Deserialize, Serialize)]
pub struct Workload {
    version: String,
    workload: ConfigVariant
}


pub fn remove_duplicates_array<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut vec: Vec<String> = Deserialize::deserialize(deserializer)?;
    vec.sort();
    vec.dedup();
    return Ok(vec);
}

// return result
pub fn read_file(filepath : PathBuf) -> Result<serde_json::Value, CustomError> {
    // read file
    let contents = match fs::read_to_string(&filepath) {
        Ok(file) => file,
        Err(error) =>  {
            match error.kind() {
                NotFound => return Err(CustomError::FileNotFound(filepath)),
                _ => return Err(CustomError::FileCouldNotBeenRead)
            }
        }
    };

    // convert file to yaml => take only the kind to know what type of Container we are reading
    let yaml: Workload = match serde_yaml::from_str::<Workload>(&contents) {
        Ok(result) => result,
        Err(err) =>  {
            println!("{}", err);
            return Err(CustomError::UnknownWorkloadKind);
        }
    };

    // check type of workload
    match yaml.workload {
        ConfigVariant::Network(ref network) => {
            // verify fields
            match verify_network(&network.egress) {
                None => (),
                Some(error) => return Err(error),
            };
            match verify_network(&network.ingress) {
                None => (),
                Some(error) => return Err(error),
            };
        }
        _ => {}
    }
    let containerstring : String = serde_yaml::to_string(&yaml).unwrap();
    let json : serde_json::Value = serde_yaml::from_str(&containerstring).unwrap();
    return Ok(json)
}