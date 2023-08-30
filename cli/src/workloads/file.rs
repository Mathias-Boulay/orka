use serde::{Serialize, Deserialize, Deserializer};
use std::fs;
use crate::workloads::container::{Container};
use crate::workloads::network::{Network, verify_network};
use thiserror::Error;
use std::io::ErrorKind::NotFound;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Error in file.")]
    FileContentError(String),
    #[error("File `{0}` not found")]
    FileNotFound(PathBuf),
    #[error("Data could not be read from file")]
    FileCouldNotBeRead,
    #[error("The ip adress `{0}` is invalid.")]
    InvalidIpAddress(String),
    #[error("The port `{0}` is outside of the allowed port range.")]
    OutsidePortRange(u32),
}

// automatically assign workload type (Container / Network) based on the defined kind
#[derive(Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum WorkloadKind {
    #[serde(rename(deserialize = "container", serialize = "Container"))]
    Container(Container),

    #[serde(rename(deserialize = "network", serialize = "Network"))]
    Network(Network),
}

#[derive(Deserialize, Serialize)]
pub struct Workload {
    version: String,
    workload: WorkloadKind
}


// remove any possible duplicates from an array
pub fn remove_duplicates_array<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut vec: Vec<String> = Deserialize::deserialize(deserializer)?;
    vec.sort();
    vec.dedup();
    return Ok(vec);
}

// - reads a given workload file 
// - verifies the fields
// - returns the workload
pub fn read_file(filepath : PathBuf) -> Result<serde_json::Value, CustomError> {
    // read file
    let contents = match fs::read_to_string(&filepath) {
        Ok(file) => file,
        Err(error) =>  {
            match error.kind() {
                NotFound => return Err(CustomError::FileNotFound(filepath)),
                _ => return Err(CustomError::FileCouldNotBeRead)
            }
        }
    };

    // convert file to yaml
    let yaml: Workload = match serde_yaml::from_str::<Workload>(&contents) {
        Ok(result) => result,
        Err(err) => return Err(CustomError::FileContentError(err.to_string())),
    };

    // check type of workload
    match yaml.workload {
        WorkloadKind::Network(ref network) => {
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
    // convert yaml to json and return it
    let containerstring : String = serde_yaml::to_string(&yaml).unwrap();
    let json : serde_json::Value = serde_yaml::from_str(&containerstring).unwrap();
    return Ok(json)
}