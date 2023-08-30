use reqwest::Response;
use serde::de::DeserializeOwned;
use crate::workloads::file::read_file;

use crate::{
    args::{
        config::{ConfigResource::ApiFqdn, GetConfig, SetConfig},
        crud::{
            CreateInstance, CreateWorkload, DeleteInstance, DeleteWorkload, GetInstance,
            GetWorkload,
        },
        OrkaCtlArgs,
    },
    DISPLAY,
};
use crate::config::Config;

pub struct Handler {
    client: reqwest::Client,
}

impl Handler {
    pub fn new() -> Self {
        Handler {
            client: reqwest::Client::new(),
        }
    }

    pub fn get_config_value(&self, args: GetConfig) {
        let config = Config::get_config_lock();
        let value: &str = match args.resource {
            ApiFqdn => &config.orka_url
        };
        DISPLAY.print_log(value);
    }

    pub fn set_config_value(&self, args: SetConfig) {}

    pub async fn create_workload(&self, args: CreateWorkload) {
        match read_file(args.file_path) {
            Ok(json) => {
                let res = self
                    .client
                    .post(Handler::get_url("workloads"))
                    .json(&json)
                    .send()
                    .await;

                let result = self
                    .generic_response_handling::<serde_json::Value>(res)
                    .await;
                if result.is_some() {
                    DISPLAY.print_log(&format!("{:?}", result.unwrap()))
                }
            },
            Err(error) => println!("{:?}", error)
        }
        
    }

    pub async fn create_instance(&self, args: CreateInstance) {
        let res = self
            .client
            .post(Handler::get_url("instance"))
            .body("")
            .send()
            .await;

        let result = self
            .generic_response_handling::<serde_json::Value>(res)
            .await;
        if result.is_some() {
            DISPLAY.print_log(&format!("{:?}", result.unwrap()))
        }
    }

    pub async fn get_workload(&self, args: GetWorkload) {
        let mut url = Handler::get_url("workload");
        if args.workload_id.is_some() {
            url += &format!("/{}", &args.workload_id.unwrap());
        }
        let res = self.client.get(url).send().await;

        let result = self
            .generic_response_handling::<serde_json::Value>(res)
            .await;

        if result.is_some() {
            DISPLAY.print_log(&format!("{:?}", result.unwrap()))
        }
    }

    pub async fn get_instance(&self, args: GetInstance) {
        let mut url = Handler::get_url("instance");
        if args.instance_id.is_some() {
            url += &format!("/{}", &args.instance_id.unwrap());
        }
        let res = self.client.get(url).send().await;

        let result = self
            .generic_response_handling::<serde_json::Value>(res)
            .await;

        if result.is_some() {
            DISPLAY.print_log(&format!("{:?}", result.unwrap()))
        }
    }

    pub async fn delete_workload(&self, args: DeleteWorkload) {
        let url = format!("{}/{}", Handler::get_url("workload"), args.workload_id);
        let res = self.client.delete(url).send().await;

        let result = self
            .generic_response_handling::<serde_json::Value>(res)
            .await;

        if result.is_some() {
            DISPLAY.print_log(&format!("{:?}", result.unwrap()))
        }
    }

    pub async fn delete_instance(&self, args: DeleteInstance) {
        let url = format!("{}/{}", Handler::get_url("instance"), args.instance_id);
        let res = self.client.delete(url).send().await;

        let result = self
            .generic_response_handling::<serde_json::Value>(res)
            .await;

        if result.is_some() {
            DISPLAY.print_log(&format!("{:?}", result.unwrap()))
        }
    }

    /// Wrapper to display common errors
    async fn generic_response_handling<T: DeserializeOwned>(
        &self,
        response: Result<Response, reqwest::Error>,
    ) -> Option<T> {
        match response {
            Err(err) => DISPLAY.print_error(&format!("{:?}", err)),
            Ok(response) => {
                let response_status = response.status();
                let response_text = response.text().await.unwrap();

                if !response_status.is_success() {
                    DISPLAY.print_error(&format!(
                        "The server returned with error {}",
                        response_status
                    ));

                    let json_err: Result<serde_json::Value, serde_json::Error> =
                        serde_json::from_str(&response_text);

                    match json_err {
                        Ok(value) => {
                            DISPLAY.print_error(&format!(
                                "Status: {} \n Message: {}",
                                value["status"], value["message"]
                            ));
                            return None;
                        }
                        Err(_) => (), // Let the default error handling
                    }
                }

                let json: Result<T, serde_json::Error> = serde_json::from_str(&response_text);
                match json {
                    Err(_) => DISPLAY.print_error("The response is not a formatted json !"),
                    Ok(json) => return Some(json),
                }
            }
        }

        return None;
    }

    fn get_url(endpoint: &str) -> String {
        //return APP_CONFIG.orka_url.clone() + endpoint;
        format!("{}{}", Config::get_config_lock().orka_url, endpoint)
    }
}
