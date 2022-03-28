use bytes::Buf;
use console::{style, Emoji};
use hyper::body::Bytes;
use hyper_tls::HttpsConnector;
use std::{
    collections::HashMap,
    error::Error,
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Index {
    pub packages: HashMap<String, Package>,
}

#[derive(Deserialize, Serialize)]
pub struct Package {
    pub version: String,
    pub image: Image,
    pub readme_url: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Image {
    Helm {
        repo_url: String,
        repo_name: String,
        chart: String,
        container_name: Option<String>,
    },
    Docker {
        compose_file: String,
    },
}

impl Index {
    pub async fn get(url: String) -> Result<Self, Box<dyn Error + 'static>> {
        let https = HttpsConnector::new();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        let res = client.get(url.parse()?).await?;
        let body = hyper::body::to_bytes(res.into_body()).await?;
        Ok(serde_json::from_reader(body.reader())?)
    }
}

impl Package {
    pub async fn install(&self) -> Result<(), Box<dyn Error + 'static>> {
        match &self.image {
            Image::Helm {
                repo_url,
                repo_name,
                chart,
                container_name,
            } => {
                {
                    let mut cmd = Command::new("helm");
                    let cmd = cmd
                        .arg("repo")
                        .arg("add")
                        .arg(repo_name)
                        .arg(repo_url)
                        .stdout(Stdio::null());
                    let pb = indicatif::ProgressBar::new_spinner();
                    pb.set_message(format!(
                        "Executing '{} {}'...",
                        style("help repo add").bold(),
                        style(repo_name).bold().blink()
                    ));
                    pb.enable_steady_tick(20);
                    cmd.output()?;
                    pb.finish_and_clear();
                    println!(
                        "{} {}Installed helm repo",
                        style("[3/?]").bold().dim(),
                        Emoji("✅️ ", ""),
                    );
                }
                {
                    let mut cmd = Command::new("helm");
                    let cmd = cmd.arg("repo").arg("update").stdout(Stdio::null());
                    let pb = indicatif::ProgressBar::new_spinner();
                    pb.set_message(format!(
                        "Executing '{}'...",
                        style("help repo update").bold(),
                    ));
                    pb.enable_steady_tick(20);
                    cmd.output()?;
                    pb.finish_and_clear();
                    println!(
                        "{} {}Updated helm repo",
                        style("[4/?]").bold().dim(),
                        Emoji("✅️ ", ""),
                    );
                }
                {
                    let mut cmd = Command::new("helm");
                    let cmd = cmd
                        .arg("helm")
                        .arg("install")
                        .arg(chart)
                        .stdout(Stdio::null());
                    match container_name {
                        Some(name) => {
                            cmd.arg("--template-name").arg(name);
                        }
                        None => {
                            cmd.arg("--generate-name");
                        }
                    };
                    let pb = indicatif::ProgressBar::new_spinner();
                    pb.set_message(format!(
                        "Executing '{} {}'...",
                        style("helm install").bold(),
                        style(chart).bold().blink()
                    ));
                    pb.enable_steady_tick(20);
                    cmd.output()?;
                    pb.finish_and_clear();
                    println!(
                        "{} {}Installed helm chart",
                        style("[5/?]").bold().dim(),
                        Emoji("✅️ ", ""),
                    );
                }
            }
            Image::Docker { compose_file } => todo!(),
        };
        Ok(())
    }
}
