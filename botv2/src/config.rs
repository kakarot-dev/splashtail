use once_cell::sync::Lazy;
use poise::serenity_prelude::UserId;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono;
use std::fs::File;

use crate::Error;

/// Global config object
pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::load().expect("Failed to load config"));

#[derive(Serialize, Deserialize, Default)]
pub struct Differs<T: Default + Clone> {
    staging: T,
    prod: T,
}

impl<T: Default + Clone> Differs<T> {
    /// Get the value for a given environment
    pub fn get_for_env(&self, env: &str) -> T {
        if env == "staging" {
            self.staging.clone()
        } else {
            self.prod.clone()
        }
    }

    /// Get the value for the current environment
    pub fn get(&self) -> T {
        self.get_for_env(&crate::ipc::argparse::MEWLD_ARGS.current_env)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct DiscordAuth {
    pub token: String,
    pub client_id: String,
    pub client_secret: String,
    pub can_use_bot: Vec<UserId>,
    pub root_users: Vec<UserId>,
}

// Object storage code

#[derive(Serialize, Deserialize)]
pub enum ObjectStorageType {
    #[serde(rename = "s3-like")]
    S3Like,
    #[serde(rename = "local")]
    Local,
}

#[derive(Serialize, Deserialize)]
pub struct ObjectStorage {
    #[serde(rename = "type")]
    pub object_storage_type: ObjectStorageType,
    pub path: String,
    pub endpoint: Option<String>,
    pub secure: Option<bool>,
    pub cdn_secure: Option<bool>,
    pub cdn_endpoint: String,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
}

pub enum ObjectStore {
    S3(object_store::aws::AmazonS3),
    Local(object_store::local::LocalFileSystem),
}

impl ObjectStore {
    pub fn get(&self) -> &dyn object_store::ObjectStore {
        match self {
            ObjectStore::S3(store) => store,
            ObjectStore::Local(store) => store,
        }
    }
}

impl ObjectStorage {
    pub fn build(&self) -> Result<ObjectStore, crate::Error> {
        match self.object_storage_type {
            ObjectStorageType::S3Like => {
                let access_key = self.access_key.as_ref().ok_or("Missing access key")?;
                let secret_key = self.secret_key.as_ref().ok_or("Missing secret key")?;
                let endpoint = self.endpoint.as_ref().ok_or("Missing endpoint")?;

                let store = object_store::aws::AmazonS3Builder::new()
                    .with_endpoint(
                        {
                            if let Some(secure) = self.secure {
                                if secure {
                                    format!("https://{}", endpoint)
                                } else {
                                    format!("http://{}", endpoint)
                                }
                            } else {
                                // Default is false
                                format!("http://{}", endpoint)
                            }
                        }
                    )
                    .with_allow_http(!self.secure.unwrap_or_default())
                    .with_bucket_name(self.path.clone())
                    .with_access_key_id(access_key)
                    .with_secret_access_key(secret_key)
                    .build()
                    .map_err(|e| format!("Failed to build S3 object store: {}", e))?;

                Ok(ObjectStore::S3(store))
            }
            ObjectStorageType::Local => {
                let store =
                    object_store::local::LocalFileSystem::new_with_prefix(self.path.clone())?;

                Ok(ObjectStore::Local(store))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Meta {
    pub postgres_url: String,
    pub bot_redis_url: String,
    pub proxy: String,
    pub support_server: String,
}

#[derive(Serialize, Deserialize)]
pub struct Sites {
    pub api: Differs<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Surreal {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub discord_auth: DiscordAuth,
    pub meta: Meta,
    pub sites: Sites,
    pub surreal: Surreal,
    pub object_storage: ObjectStorage,

    #[serde(skip)]
    /// Setup by load() for statistics
    pub bot_start_time: i64,
}

impl Config {
    pub fn load() -> Result<Self, Error> {
        // Open config.yaml from parent directory
        let file = File::open("../config.yaml");

        match file {
            Ok(file) => {
                // Parse config.yaml
                let mut cfg: Config = serde_yaml::from_reader(file)?;

                cfg.bot_start_time = chrono::Utc::now().timestamp();

                // Return config
                Ok(cfg)
            }
            Err(e) => {
                // Print error
                println!("config.yaml could not be loaded: {}", e);

                // Exit
                std::process::exit(1);
            }
        }
    }
}
