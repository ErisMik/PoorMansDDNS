use clap::Parser;
use clokwerk::{Scheduler, TimeUnits};
use config;
use reqwest;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json;
use std::thread;
use std::time::Duration;

const HEALTHCHECK_HOST: &str = "https://hc-ping.com";
const IPFY_URL: &str = "https://api.ipify.org";
const CLOUDFLARE_HOST: &str = "https://api.cloudflare.com/client/v4";

#[derive(Clone, Debug, Deserialize)]
struct Settings {
    healthcheck_uuid: String,
    dns_cloudflare_email: String,
    dns_cloudflare_api_key: String,
}

impl Settings {
    fn new(config: config::Config) -> Result<Self, config::ConfigError> {
        config.try_deserialize()
    }
}

fn get_current_ipaddr() -> String {
    let result = reqwest::blocking::get(IPFY_URL).unwrap();
    return result.text().unwrap();
}

#[derive(Clone, Debug, Deserialize)]
struct CloudflareResponse<T> {
    success: bool,
    result: Vec<T>,
}

#[derive(Clone, Debug, Deserialize)]
struct CloudflareZonesResponse {
    id: String,
    name: String,
}

#[derive(Clone, Debug, Deserialize)]
struct CloudflareDNSListResponse {
    id: String,
    r#type: String,
    name: String,
    content: String,
}

#[derive(Clone, Debug, Serialize)]
struct CloudflareDNSPatchRequest {
    content: String,
}

fn cloudflare_get_request(
    auth_key: &str,
    auth_email: &str,
    url: &str,
) -> reqwest::blocking::Response {
    let client = reqwest::blocking::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert("X-Auth-Key", HeaderValue::from_str(auth_key).unwrap());
    headers.insert("X-Auth-Email", HeaderValue::from_str(auth_email).unwrap());

    return client.get(url).headers(headers).send().unwrap();
}

fn cloudflare_patch_request(
    auth_key: &str,
    auth_email: &str,
    url: &str,
    body: String,
) -> reqwest::blocking::Response {
    let client = reqwest::blocking::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert("X-Auth-Key", HeaderValue::from_str(auth_key).unwrap());
    headers.insert("X-Auth-Email", HeaderValue::from_str(auth_email).unwrap());

    return client
        .patch(url)
        .headers(headers)
        .body(body)
        .send()
        .unwrap();
}

fn update_dns_ipaddrs(auth_key: &str, auth_email: &str, new_ipv4: String) {
    let zone_list_response =
        cloudflare_get_request(auth_key, auth_email, &format!("{}/zones/", CLOUDFLARE_HOST))
            .json::<CloudflareResponse<CloudflareZonesResponse>>()
            .unwrap();

    for zone in zone_list_response.result {
        let record_list_response = cloudflare_get_request(
            auth_key,
            auth_email,
            &format!("{}/zones/{}/dns_records", CLOUDFLARE_HOST, zone.id),
        )
        .json::<CloudflareResponse<CloudflareDNSListResponse>>()
        .unwrap();

        for dns_record in record_list_response.result {
            match dns_record.r#type.as_str() {
                "A" => {
                    let payload = CloudflareDNSPatchRequest {
                        content: new_ipv4.clone(),
                    };
                    cloudflare_patch_request(
                        auth_key,
                        auth_email,
                        &format!(
                            "{}/zones/{}/dns_records/{}",
                            CLOUDFLARE_HOST, zone.id, dns_record.id
                        ),
                        serde_json::to_string(&payload).unwrap(),
                    );
                }
                "AAAA" => {}
                _ => {}
            }
        }
    }
}

fn sync_domain(settings: Settings) {
    let base_url = format!("{}/{}", HEALTHCHECK_HOST, settings.healthcheck_uuid);
    reqwest::blocking::get(&format!("{}/start", base_url)).unwrap();

    let current_ipaddr = get_current_ipaddr();
    update_dns_ipaddrs(
        &settings.dns_cloudflare_api_key,
        &settings.dns_cloudflare_email,
        current_ipaddr,
    );

    reqwest::blocking::get(&format!("{}/0", base_url)).unwrap();
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[clap(short, long, default_value_t = String::from("pmddns.toml"))]
    config: String,
}

fn main() {
    let args = Cli::parse();
    let builder = config::Config::builder().add_source(config::File::with_name(&args.config));
    let config = builder.build().unwrap();

    let settings = Settings::new(config).unwrap();

    let mut scheduler = Scheduler::new();
    scheduler
        .every(15.minute())
        .run(move || sync_domain(settings.clone()));
    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_millis(1000));
    }
}
