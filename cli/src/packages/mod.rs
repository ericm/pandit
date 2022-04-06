use bytes::Buf;
use console::{style, Emoji};
use hyper::body::Bytes;
use hyper_tls::HttpsConnector;
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::Write,
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
};

use serde::{Deserialize, Serialize};

use crate::Args;

#[derive(Deserialize, Serialize)]
pub struct Index {
    pub packages: HashMap<String, Package>,
}

#[derive(Deserialize, Serialize)]
pub struct Package {
    pub version: String,
    pub proto_url: String,
    pub image: Option<Image>,
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

macro_rules! handle_output {
    ($value:expr) => {
        let output = $value?;
        if !output.status.success() {
            return Err(String::from_utf8(output.stderr)?.into());
        }
    };
}

impl Package {
    pub async fn install(
        &self,
        name: String,
        proto_path: &String,
    ) -> Result<(), Box<dyn Error + 'static>> {
        match &self.image {
            Some(img) => match img {
                Image::Helm {
                    repo_url,
                    repo_name,
                    chart,
                    container_name,
                } => {
                    {
                        let mut cmd = Command::new("helm");
                        let cmd = cmd.arg("repo").arg("add").arg(repo_name).arg(repo_url);
                        let pb = indicatif::ProgressBar::new_spinner();
                        pb.set_message(format!(
                            "Executing '{} {}'...",
                            style("help repo add").bold(),
                            style(repo_name).bold().blink()
                        ));
                        pb.enable_steady_tick(20);
                        handle_output!(cmd.output());
                        pb.finish_and_clear();
                        println!(
                            "{} {}Installed helm repo",
                            style("[3/?]").bold().dim(),
                            Emoji("✅️ ", ""),
                        );
                    }
                    {
                        let mut cmd = Command::new("helm");
                        let cmd = cmd.arg("repo").arg("update");
                        let pb = indicatif::ProgressBar::new_spinner();
                        pb.set_message(format!(
                            "Executing '{}'...",
                            style("help repo update").bold(),
                        ));
                        pb.enable_steady_tick(20);
                        handle_output!(cmd.output());
                        pb.finish_and_clear();
                        println!(
                            "{} {}Updated helm repo",
                            style("[4/?]").bold().dim(),
                            Emoji("✅️ ", ""),
                        );
                    }
                    {
                        let mut cmd = Command::new("helm");
                        let cmd = cmd.arg("install").arg(chart);
                        match container_name {
                            Some(name) => {
                                cmd.arg("--name-template").arg(name);
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
                        handle_output!(cmd.output());
                        pb.finish_and_clear();
                        println!(
                            "{} {}Installed helm chart",
                            style("[5/?]").bold().dim(),
                            Emoji("✅️ ", ""),
                        );
                    }
                }
                Image::Docker { compose_file } => todo!(),
            },
            None => {}
        }

        let https = HttpsConnector::new();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        let res = client.get(self.proto_url.parse()?).await?;
        let body = hyper::body::to_bytes(res.into_body()).await?;
        let path = PathBuf::from(&proto_path)
            .join(name)
            .with_extension("proto")
            .to_str()
            .unwrap()
            .to_string();
        let mut save_file = File::create(&path)?;
        save_file.write_all(&body.to_vec()[..])?;
        println!(
            "{} {}Installed proto to '{}'",
            style("[?/?]").bold().dim(),
            Emoji("✅️ ", ""),
            style(path).green(),
        );
        Ok(())
    }
}
