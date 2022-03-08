extern crate clap;
extern crate clokwerk;
extern crate config;
extern crate reqwest;
extern crate retry;

#[macro_use]
extern crate serde_derive;

use clap::{App, Arg};
use clokwerk::{Scheduler, TimeUnits};
use retry::{delay, retry};
use std::collections::HashMap;
use std::process::Command;
use std::thread;
use std::time::Duration;

const HEALTHCHECK_HOST: &str = "https://hc-ping.com";

#[derive(Clone, Debug, Deserialize)]
struct DomainSettings {
    domain: String,
    healthcheck_uuid: String
}

#[derive(Clone, Debug, Deserialize)]
struct Settings {
    domains: HashMap<String, DomainSettings>,
}

impl Settings {
    fn new(config: config::Config) -> Result<Self, config::ConfigError> {
        config.try_into()
    }
}

fn get_dns_ipaddr() {
}

fn get_current_ipaddr() {
}

fn sync_domain(name: String, domain_settings: DomainSettings) {
    let base_url = format!("{}/{}", HEALTHCHECK_HOST, domain_settings.healthcheck_uuid);
    reqwest::blocking::get(&format!("{}/start", base_url)).unwrap();

    let dns_ipaddr = get_current_domain_ipaddr();
    let current_ipaddr = get_current_ipaddr();

    println!("Syncing {} with command {:?}", name, command);
    let result = retry(delay::Fixed::from_millis(300000).take(5), || {
        let mut child = command.spawn().expect("Command failed to start");
        let result = child.wait().unwrap();

        let return_code = result.code().unwrap();
        match return_code {
            0 => Ok(return_code),
            _ => Err(return_code),
        }
    });

    match result {
        Ok(return_code) => {
            reqwest::blocking::get(&format!("{}/{}", base_url, return_code)).unwrap()
        }
        Err(error) => match error {
            retry::Error::Operation { error, .. } => {
                reqwest::blocking::get(&format!("{}/{}", base_url, error)).unwrap()
            }
            _ => reqwest::blocking::get(&format!("{}/{}", base_url, "fail")).unwrap(),
        },
    };
}

fn main() {
    let matches = App::new("Poorman's DDNS")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .default_value("matt.toml")
                .takes_value(true),
        )
        .get_matches();

    let config_filename = matches.value_of("config").unwrap();
    let mut config = config::Config::new();
    config
        .merge(config::File::with_name(config_filename))
        .unwrap();
    let settings = Settings::new(config).unwrap();

    let mut scheduler = Scheduler::new();

    settings.domains.into_iter().for_each(|(name, domain_settings)| {
        scheduler
            .every(5.minute())
            .run(|| sync_domain(name, domain_settings));
    });

    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_millis(1000));
    }
}
