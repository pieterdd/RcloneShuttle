use serde::{Deserialize, Serialize};
use std::{
    process::{Command, Stdio},
    str::from_utf8,
};
use time::OffsetDateTime;

use crate::path_tools::RclonePath;

#[derive(Serialize, Deserialize)]
pub struct ImportedFileListing {
    #[serde(rename = "Path")]
    pub path: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Size")]
    pub size: i64,
    #[serde(rename = "MimeType")]
    pub mime_type: String,
    #[serde(with = "time::serde::rfc3339", rename = "ModTime")]
    pub mod_time: OffsetDateTime,
    #[serde(rename = "IsDir")]
    pub is_dir: bool,
    #[serde(rename = "IsBucket")]
    pub is_bucket: Option<bool>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct RcloneFileListing {
    pub path: RclonePath,
    pub name: String,
    pub size: i64,
    pub mime_type: String,
    pub mod_time: OffsetDateTime,
    pub is_dir: bool,
    pub is_bucket: Option<bool>,
}

impl RcloneFileListing {
    fn from(imported_file_listing: &ImportedFileListing, parent_path: &RclonePath) -> Self {
        Self {
            path: parent_path.join(&imported_file_listing.path),
            name: imported_file_listing.name.clone(),
            size: imported_file_listing.size,
            mime_type: imported_file_listing.mime_type.clone(),
            mod_time: imported_file_listing.mod_time,
            is_dir: imported_file_listing.is_dir,
            is_bucket: imported_file_listing.is_bucket,
        }
    }

    pub fn formatted_size(&self) -> Option<String> {
        if self.size == -1 || self.size == 0 {
            None
        } else {
            Some(size_format::SizeFormatterSI::new(self.size as u64).to_string())
        }
    }
}

#[derive(Debug)]
pub enum MkdirError {
    NotAvailableHere,
    Generic(String),
}

#[derive(Debug, Clone)]
pub struct RcloneClient {
    password: Option<String>,
    custom_config_path: Option<String>,
}

impl RcloneClient {
    pub fn new(
        password: Option<String>,
        custom_config_path: Option<String>,
    ) -> Result<Self, String> {
        let client = RcloneClient {
            password,
            custom_config_path,
        };
        client.list_remotes().map(|_| client)
    }

    fn build_command(&self) -> Command {
        let mut cmd = Command::new("rclone");
        if let Some(password) = self.password.clone() {
            cmd.env("RCLONE_PASSWORD_COMMAND", format!("echo \"{}\"", password))
                .stdin(Stdio::null());
        }
        if let Some(custom_config_path) = self.custom_config_path.clone() {
            cmd.args([format!("--config={}", &custom_config_path)]);
        }
        cmd
    }

    pub fn is_password_required(custom_config_path: &Option<String>) -> Result<bool, String> {
        let mut cmd = Command::new("rclone");
        cmd.args(["config", "show"]).stdin(Stdio::null());
        if let Some(custom_config_path) = custom_config_path.clone() {
            cmd.args([format!("--config={}", &custom_config_path)]);
        }
        let output = cmd
            .output()
            .map_err(|_| "Password requirement check could not start")?;
        Ok(!output.status.success())
    }

    pub fn list_remotes(&self) -> Result<Vec<String>, String> {
        let output = self
            .build_command()
            .args(["listremotes", "--use-json-log"])
            .output()
            .map_err(|_| "Command did not start")?;

        if output.status.success() {
            let raw_remotes = from_utf8(&output.stdout)
                .expect("Rclone output encode to UTF8 failed")
                .to_owned();
            let remotes = Vec::from_iter(
                raw_remotes
                    .trim()
                    .split('\n')
                    .filter(|r| *r != "")
                    .map(Into::into),
            );
            Ok(remotes)
        } else {
            Err(format!("Rclone command failed - {}", output.status))
        }
    }

    pub fn ls(&self, path: &RclonePath) -> Result<Vec<RcloneFileListing>, String> {
        let output = self
            .build_command()
            .args(["lsjson", &path.to_string()])
            .output()
            .map_err(|_| "Command did not start")?;

        if output.status.success() {
            let utf8_output = from_utf8(&output.stdout)
                .expect("Rclone output encode to UTF8 failed")
                .to_owned();
            let imported_listings: Vec<ImportedFileListing> = serde_json::from_str(&utf8_output)
                .unwrap_or_else(|_| panic!("Could not decode {}", utf8_output));
            let rclone_listings = imported_listings
                .iter()
                .map(|l| RcloneFileListing::from(l, path))
                .collect();
            Ok(rclone_listings)
        } else {
            Err(format!(
                "Rclone command failed with {}\n\n{}",
                output.status,
                from_utf8(&output.stderr).unwrap(),
            ))
        }
    }

    pub fn copy(&self, source_path: &RclonePath, target_path: &RclonePath) -> Result<(), String> {
        let output = self
            .build_command()
            .args(["copyto", &source_path.to_string(), &target_path.to_string()])
            .output()
            .map_err(|_| "Command did not start")?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Copy failed with {}\n\n{}",
                output.status,
                from_utf8(&output.stderr).unwrap(),
            ))
        }
    }

    pub fn mv(&self, source_path: &RclonePath, target_path: &RclonePath) -> Result<(), String> {
        let target_directory = target_path.resolve_to_parent();
        let output = self
            .build_command()
            .args([
                "move",
                &source_path.to_string(),
                &target_directory.to_string(),
            ])
            .output()
            .map_err(|_| "Command did not start")?;

        if output.status.success() {
            Ok(())
        } else {
            println!("{}", from_utf8(&output.stderr).unwrap());
            Err(format!(
                "Copy failed with {}\n\n{}",
                output.status,
                from_utf8(&output.stderr).unwrap(),
            ))
        }
    }

    pub fn rm(&self, path: &RclonePath) -> Result<(), String> {
        let output = self
            .build_command()
            .args(["delete", &path.to_string()])
            .output()
            .map_err(|_| "Command did not start")?;

        match output.status.success() {
            true => Ok(()),
            false => Err(format!(
                "Rclone command failed with {}\n\n{}",
                output.status,
                from_utf8(&output.stderr).unwrap(),
            )),
        }
    }

    pub fn mkdir(&self, path: &RclonePath) -> Result<(), MkdirError> {
        let output = self
            .build_command()
            .args(["mkdir", &path.to_string()])
            .output()
            .map_err(|_| MkdirError::Generic(String::from("Command did not start")))?;

        if !output.status.success() {
            return Err(MkdirError::Generic(format!(
                "Rclone command failed with {}\n\n{}",
                output.status,
                from_utf8(&output.stderr).unwrap()
            )));
        }

        let stderr_str = from_utf8(&output.stderr).expect("UTF8 decode failed");
        match stderr_str.contains(
            "Warning: running mkdir on a remote which can't have empty directories does nothing",
        ) {
            true => Err(MkdirError::NotAvailableHere),
            false => Ok(()),
        }
    }
}
