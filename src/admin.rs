extern crate hyper;

use multipart::client::Multipart;
use url::Url;

use hyper::net::Streaming;
use hyper::client::request::Request;
use hyper::header::Connection;

use super::client::PdbClient;
use super::net::Auth;
use super::utils::Result;

/// POSTs a multipart request to PuppetDB for importing an archive.
pub fn post_import(pdb_client: &PdbClient, path: String) -> Result {
    // Import and export are not prepared to use token auth
    let server_url: String = pdb_client.server_urls[0].clone();
    let url = Url::parse(&(server_url + "/pdb/admin/v1/archive")).unwrap();
    let mut multipart: Multipart<Request<Streaming>> = Auth::multipart(&pdb_client.auth, url);
    multipart.write_file("archive", &path);
    multipart.send()
}

pub fn get_export(pdb_client: &PdbClient, anonymization: String) -> Result {
    // Import and export are not prepared to use token auth
    let server_url: String = pdb_client.server_urls[0].clone();
    Auth::client(&pdb_client.auth)
        .get(&(server_url + "/pdb/admin/v1/archive"))
        .body(&("anonymization=".to_string() + &anonymization))
        .header(Connection::close())
        .send()
}

use std::collections::BTreeMap;
use std::io::Read;
use rustc_serialize::json::{self, ToJson};

fn build_status_error_json(e: String) -> json::Json {
    let mut error_map = BTreeMap::new();
    error_map.insert("error".to_string(), e.to_json());
    json::Json::Object(error_map)
}

fn build_status_json(resp: Result) -> json::Json {
    match resp {
        Ok(mut r) => {
            match r.status {
                hyper::Ok => {
                    let mut b = json::Builder::new(r.bytes().map(|c| c.unwrap() as char));
                    match b.build() {
                        Ok(r) => r,
                        Err(e) => {
                            let msg = format!("Unable to build JSON object from server: {}", e);
                            build_status_error_json(msg)
                        }
                    }
                }
                _ => {
                    let mut temp = String::new();
                    let msg = match r.read_to_string(&mut temp) {
                        Err(x) => format!("Unable to read response from server: {}", x),
                        _ => temp,
                    };
                    build_status_error_json(msg)
                }
            }
        }
        Err(e) => build_status_error_json(e.to_string()),
    }
}

pub fn get_status(pdb_client: &PdbClient) {
    let mut map = BTreeMap::new();
    let cli = Auth::client(&pdb_client.auth);

    for server_url in pdb_client.server_urls.clone() {
        let mut req = cli.get(&(server_url.clone() + "/status/v1/services"))
                         .header(Connection::close());
        if let Some(auth) = Auth::auth_header(&pdb_client.auth) {
            req = req.header(auth)
        };
        let res = req.send();
        map.insert(server_url, build_status_json(res));
    }

    println!("{}", json::as_pretty_json(&json::Json::Object(map)));
}
