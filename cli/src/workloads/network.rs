use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::workloads::file::{CustomError, remove_duplicates_array};
use regex::Regex;
use validator::{Validate};


#[derive(Serialize, Deserialize)]
pub struct Network {
    name: String,
    #[serde(rename = "allowService", deserialize_with="remove_duplicates_array")]
    allowservice: Vec<String>,
    #[serde(default)]
    pub egress: Vec<HashMap<String, IpAdress>>,
    #[serde(default)]
    pub ingress: Vec<HashMap<String, IpAdress>>,
}



#[derive(Validate, Serialize, Deserialize)]
pub struct IpAdress {
    #[validate(range(min = 0, max = 32))]
    #[serde(default = "max_mask")]
    mask: u32,
    #[serde(default)]
    ports: Vec<String>
}

fn max_mask() -> u32{
    return 32;
}



pub fn verify_network(networks: &Vec<HashMap<String, IpAdress>>) -> Option<CustomError> {
    let re = Regex::new(r"^([0-9]{1,3})\.([0-9]{1,3})\.([0-9]{1,3})\.([0-9]{1,3})$").unwrap();
    for hashmap in networks {
        for (key, ip_address) in hashmap {
            // verify ip address
            let Some(_) = re.captures(key) else {
                return Some(CustomError::InvalidIpAddress(key.to_string()));
            };

            // verify ports
            for port in &ip_address.ports {
                let ports = port.split('-');
                for p in ports {
                    let port_number: u32 = p.parse().unwrap();
                        if port_number > 65535 {
                            return Some(CustomError::OutsidePortRange(port_number));
                        }
                }
            }
        }
    }
    return None;
}