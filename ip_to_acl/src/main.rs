use ip_network::IpNetwork;
use reqwest;
use reqwest::StatusCode;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

const API_HOST: &str = "https://api.fastly.com";

// Supply Fastly API Token
const API_KEY: &str = "XXXXXXXXXXXXXXXXXXXXXX";

// Supply Fastly ServiceID that the ACL should be applied on
const SERVICE_ID: &str = "XXXXXXXXXXXXXXXXXXXXXX";

// Supply path to local file where IPs are stored
const IP_LIST_PATH: &str = "XXXXXXXXXXXXXXXXXXXXXX";

pub type RootVersionProperties = Vec<VersionProperties>;
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionProperties {
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "deleted_at")]
    pub deleted_at: Value,
    pub active: bool,
    pub comment: String,
    pub deployed: bool,
    pub locked: bool,
    pub number: i64,
    pub staging: bool,
    pub testing: bool,
    #[serde(rename = "service_id")]
    pub service_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloneVersionProperties {
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "deleted_at")]
    pub deleted_at: Value,
    pub active: bool,
    pub comment: String,
    pub deployed: bool,
    pub locked: bool,
    pub number: i64,
    pub staging: bool,
    pub testing: bool,
    #[serde(rename = "service_id")]
    pub service_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AclProperties {
    pub version: String,
    pub name: String,
    #[serde(rename = "service_id")]
    pub service_id: String,
    pub id: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "deleted_at")]
    pub deleted_at: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    #[serde(rename = "ip_list")]
    pub ip_list: Vec<String>,
}

fn active_version_fetch(c: &reqwest::blocking::Client) -> i64 {
    // Complier is requiring an initialized value - so starting with 0 which is invalid version number
    let mut active_version = 0;

    // Retrieve current active version
    let version_check_resp = c
        .get(format!("{}/service/{}/version", API_HOST, SERVICE_ID))
        .header("Fastly-Key", API_KEY)
        .send();

    // Handle failure cases
    match version_check_resp {
        Ok(r) => {
            // Make sure we got a successful response back from the Fastly API
            match r.status() {
                StatusCode::OK => {
                    // Serialize the response body into version property struct
                    let version_body = r.text().unwrap();
                    let des_version_body: RootVersionProperties =
                        serde_json::from_str(&version_body).unwrap();

                    // Loop through the versions in order to find the active one
                    for versions in des_version_body {
                        if versions.active == true {
                            active_version = versions.number;
                        }
                    }
                }
                _ => {
                    format!(
                        "{}{:?}. Please refine request parameters and try again.",
                        r.status(),
                        r.text()
                    );
                }
            }
        }
        Err(e) => {
            panic!(
                "Error: {}. Please refine request parameters and try again.",
                e
            );
        }
    };

    // Validate result
    match active_version {
        0 => {
            panic!("No version active. Please activate a version and try again.");
        }
        _ => (),
    }

    // Return active version number to main
    active_version
}

fn clone_active_version(c: &reqwest::blocking::Client, av: i64) -> i64 {
    // Complier is requiring an initialized value - so starting with 0 which is invalid version number
    let mut draft_version_num = 0;

    // Clone the active version
    let create_draft_resp = c
        .put(format!(
            "{}/service/{}/version/{}/clone",
            API_HOST, SERVICE_ID, av
        ))
        .header("Fastly-Key", API_KEY)
        .send();

    // Handle failure cases
    match create_draft_resp {
        Ok(r) => {
            // Make sure we got a successful response back from the Fastly API
            match r.status() {
                StatusCode::OK => {
                    let clone_resp_body = r.text().unwrap();
                    let des_clone_resp_body: CloneVersionProperties =
                        serde_json::from_str(&clone_resp_body).unwrap();
                    draft_version_num = des_clone_resp_body.number;
                }
                _ => {
                    format!(
                        "{}{:?}. Please refine request parameters and try again.",
                        r.status(),
                        r.text()
                    );
                }
            }
        }
        Err(e) => {
            panic!(
                "Error: {}. Please refine request parameters and try again.",
                e
            );
        }
    };

    // Validate result
    match draft_version_num {
        0 => {
            panic!("No draft version available. Please activate a version and try again.");
        }
        _ => (),
    }

    println!("Working draft version: {}", &draft_version_num);
    draft_version_num
}

fn create_acl(c: &reqwest::blocking::Client, dv: i64) -> String {
    // Once again pre-defining to avoid the compiler not liking a potential empty value
    let mut acl_id = format!("null");
    // Create ACL
    let acl_init_resp = c
        .post(format!(
            "{}/service/{}/version/{}/acl",
            API_HOST, SERVICE_ID, &dv
        ))
        .body("name=placeholder_name")
        .header("Fastly-Key", API_KEY)
        .send();

    // Handle failure cases
    match acl_init_resp {
        Ok(r) => match r.status() {
            StatusCode::OK => {
                let acl_create_resp_body = r.text().unwrap();
                let des_acl_create_resp_body: AclProperties =
                    serde_json::from_str(&acl_create_resp_body).unwrap();
                acl_id = des_acl_create_resp_body.id;
            }
            _ => {
                format!(
                    "{}{:?}. Please refine request parameters and try again.",
                    r.status(),
                    r.text()
                );
            }
        },
        Err(e) => {
            panic!(
                "Error: {}. Please refine request parameters and try again.",
                e
            );
        }
    };

    // Validate result
    match acl_id.as_str() {
        "null" => {
            panic!("No acl id available. Please activate a version and try again.");
        }
        _ => (),
    }
    acl_id
}

fn read_ips_from_file<P: AsRef<Path>>(path: P) -> Result<Root, Box<dyn Error>> {
    // Open the file in read-only mode with buffer
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of Root
    let contents = serde_json::from_reader(reader)?;

    // Return contents serialized as Root
    Ok(contents)
}

fn main() {
    // Create a client in order to make use of connection resuse
    let client = reqwest::blocking::Client::new();

    // Retrieve active version for service
    let active_version = active_version_fetch(&client);

    // Clone the active version
    let draft_version = clone_active_version(&client, active_version);

    // Create ACL on new draft version
    let acl_id = create_acl(&client, draft_version);

    // Retrieve contents from local file
    let contents = read_ips_from_file(IP_LIST_PATH).unwrap();

    // Serialize IP entries to a vector of Strings
    let ips = contents.ip_list;

    // Instantiate failed_ip vector for reporting later
    let mut failed_ips: Vec<String> = vec![format!("Failed IPs:")];

    // Kickoff the loop that will iterate through the vector
    // and push each entry to the ACL we've created
    for entry in ips {
        println!("{}", &entry);
        // Attempt to parse IP with netmask
        let ip_parse = IpNetwork::from_str_truncate(&entry);
        match ip_parse {
            // We have a netmask
            Ok(ip) => {
                // Build the request body including a netmask
                let req_body = format!(
                    r#"{{"comment":"","created_at":null,"ip":"{}","negated":false,"subnet":"{}","acl_id":"{}","service_id":null}}"#,
                    ip.network_address(),
                    ip.netmask(),
                    &acl_id
                );
                // Build and send the request
                let acl_entry_resp = client
                    .post(format!(
                        "{}/service/{}/acl/{}/entry",
                        API_HOST, SERVICE_ID, acl_id
                    ))
                    .body(req_body)
                    .header("Fastly-Key", API_KEY)
                    .header("content-type", "application/json; charset=UTF-8")
                    .send();

                // If we experience a timeout or a network issue, send the IP we tried to add to the failed IPs list
                match acl_entry_resp {
                    Ok(r) => {
                        // If the response from the Fastly API is anything but a 200, send the IP to the failed IPs list
                        match r.status() {
                            StatusCode::OK => (),
                            _ => failed_ips.push(entry),
                        };
                    }
                    Err(_e) => failed_ips.push(entry),
                }
            }
            // We have no netmask
            Err(_no_netmask) => {
                // Build the request body without a netmask
                let req_body = format!(
                    r#"{{"comment":"","created_at":null,"ip":"{}","negated":false,"subnet":null,"acl_id":"{}","service_id":null}}"#,
                    entry, &acl_id
                );
                // Build and send the request
                let acl_entry_resp = client
                    .post(format!(
                        "{}/service/{}/acl/{}/entry",
                        API_HOST, SERVICE_ID, acl_id
                    ))
                    .body(req_body)
                    .header("Fastly-Key", API_KEY)
                    .header("content-type", "application/json; charset=UTF-8")
                    .send();

                // If we experience a timeout or a network issue, send the IP to the failed IPs list
                match acl_entry_resp {
                    Ok(r) => {
                        // If the response from the Fastly API is anything but a 200, send the IP to the failed IPs list
                        match r.status() {
                            StatusCode::OK => (),
                            _ => failed_ips.push(entry),
                        };
                    }
                    Err(_e) => failed_ips.push(entry),
                }
            }
        };
    }
    println!("ACL migration complete!");
    println!("IPs that failed upload: {:?}", failed_ips);
}
