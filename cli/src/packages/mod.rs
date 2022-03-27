use bytes::Buf;
use hyper::body::Bytes;
use hyper_tls::HttpsConnector;
use std::{collections::HashMap, error::Error};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Packages {
    packages: HashMap<String, Package>,
}

#[derive(Deserialize, Serialize)]
pub struct Package {
    version: String,
    image: Image,
    readme_url: Option<String>,
    logo_url: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Image {
    Helm {
        repo: String,
        chart: String,
        container_name: Option<String>,
    },
    Docker {
        compose_file: String,
    },
}

impl Packages {
    pub async fn get(url: String) -> Result<Self, Box<dyn Error + 'static>> {
        let https = HttpsConnector::new();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        let res = client.get(url.parse()?).await?;
        let body = hyper::body::to_bytes(res.into_body()).await?;
        Ok(serde_json::from_reader(body.reader())?)
    }
}
