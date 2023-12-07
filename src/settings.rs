/*
 * Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::env;
use std::fs;
use std::path::Path;

use config::{Config, ConfigError, Environment, File};
use log::warn;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub port: u32,
    pub domain: String,
    pub ip: String,
    pub proxy_has_tls: bool,
    pub workers: Option<usize>,
}

impl Server {
    #[cfg(not(tarpaulin_include))]
    pub fn get_ip(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub debug: bool,
    pub cache: Option<String>,
    pub server: Server,
    pub source_code: String,
}

#[cfg(not(tarpaulin_include))]
impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::builder();

        // setting default values
        const CURRENT_DIR: &str = "./config/default.toml";
        const ETC: &str = "/etc/libmedium/config.toml";

        if let Ok(path) = env::var("LIBMEDIUM") {
            s = s.add_source(File::with_name(&path));
        } else if Path::new(CURRENT_DIR).exists() {
            // merging default config from file
            s = s.add_source(File::with_name(CURRENT_DIR));
        } else if Path::new(ETC).exists() {
            s = s.add_source(File::with_name(ETC));
        } else {
            log::warn!("configuration file not found");
        }

        s = s.add_source(Environment::with_prefix("PAGES").separator("__"));

        match env::var("PORT") {
            Ok(val) => {
                s = s.set_override("server.port", val).unwrap();
            }
            Err(e) => warn!("couldn't interpret PORT: {}", e),
        }

        let mut settings: Settings = s.build()?.try_deserialize::<Settings>()?;
        settings.check_url();

        if settings.cache.is_none() {
            let tmp = env::temp_dir().join("libmedium_cache_path");
            if !tmp.exists() {
                fs::create_dir_all(&tmp).unwrap()
            }
            settings.cache = Some(tmp.to_str().unwrap().to_string())
        }

        let cache_path = settings.cache.as_ref().unwrap();
        let cache_path = Path::new(&cache_path);
        if !cache_path.exists() {
            fs::create_dir(cache_path).unwrap();
        }
        if !cache_path.is_dir() {
            panic!(
                "Cache path {} must be a directory",
                &settings.cache.as_ref().unwrap()
            );
        }
        Ok(settings)
    }

    #[cfg(not(tarpaulin_include))]
    fn check_url(&self) {
        Url::parse(&self.source_code).expect("Please enter a URL for source_code in settings");
    }
}
