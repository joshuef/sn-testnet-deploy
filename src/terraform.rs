// Copyright (c) 2023, MaidSafe.
// All rights reserved.
//
// This SAFE Network Software is licensed under the BSD-3-Clause license.
// Please see the LICENSE file for more details.

use crate::error::Result;
use crate::run_external_command;
use crate::CloudProvider;
#[cfg(test)]
use mockall::automock;
use std::path::PathBuf;

/// Provides an interface which corresponds to Terraform commands.
///
/// To keep things simple, each subcommand will be its own function.
///
/// This trait exists for unit testing: it enables testing behaviour without actually calling the
/// Terraform process.
#[cfg_attr(test, automock)]
pub trait TerraformRunnerInterface {
    fn apply(&self, vars: Vec<(String, String)>) -> Result<()>;
    fn destroy(&self) -> Result<()>;
    fn init(&self) -> Result<()>;
    fn workspace_delete(&self, name: &str) -> Result<()>;
    fn workspace_list(&self) -> Result<Vec<String>>;
    fn workspace_new(&self, name: &str) -> Result<()>;
    fn workspace_select(&self, name: &str) -> Result<()>;
}

pub struct TerraformRunner {
    pub binary_path: PathBuf,
    pub provider: CloudProvider,
    pub working_directory_path: PathBuf,
    pub state_bucket_name: String,
}

impl TerraformRunnerInterface for TerraformRunner {
    fn apply(&self, vars: Vec<(String, String)>) -> Result<()> {
        let mut args = vec!["apply".to_string(), "-auto-approve".to_string()];
        for var in vars.iter() {
            args.push("-var".to_string());
            args.push(format!("{}={}", var.0, var.1));
        }
        run_external_command(
            self.binary_path.clone(),
            self.working_directory_path.clone(),
            args,
            false,
        )?;
        Ok(())
    }

    fn destroy(&self) -> Result<()> {
        run_external_command(
            self.binary_path.clone(),
            self.working_directory_path.clone(),
            vec!["destroy".to_string(), "-auto-approve".to_string()],
            false,
        )?;
        Ok(())
    }

    fn init(&self) -> Result<()> {
        let args = vec![
            "init".to_string(),
            "-backend-config".to_string(),
            format!("bucket={}", self.state_bucket_name),
        ];
        run_external_command(
            self.binary_path.clone(),
            self.working_directory_path.clone(),
            args,
            false,
        )?;
        Ok(())
    }

    fn workspace_delete(&self, name: &str) -> Result<()> {
        run_external_command(
            self.binary_path.clone(),
            self.working_directory_path.clone(),
            vec![
                "workspace".to_string(),
                "delete".to_string(),
                name.to_string(),
            ],
            true,
        )?;
        Ok(())
    }

    fn workspace_list(&self) -> Result<Vec<String>> {
        let output = run_external_command(
            self.binary_path.clone(),
            self.working_directory_path.clone(),
            vec!["workspace".to_string(), "list".to_string()],
            true,
        )?;
        let workspaces: Vec<String> = output
            .into_iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().trim_start_matches('*').trim().to_string())
            .collect();
        Ok(workspaces)
    }

    fn workspace_new(&self, name: &str) -> Result<()> {
        run_external_command(
            self.binary_path.clone(),
            self.working_directory_path.clone(),
            vec!["workspace".to_string(), "new".to_string(), name.to_string()],
            false,
        )?;
        Ok(())
    }

    fn workspace_select(&self, name: &str) -> Result<()> {
        run_external_command(
            self.binary_path.clone(),
            self.working_directory_path.clone(),
            vec![
                "workspace".to_string(),
                "select".to_string(),
                name.to_string(),
            ],
            false,
        )?;
        Ok(())
    }
}

impl TerraformRunner {
    pub fn new(
        binary_path: PathBuf,
        working_directory: PathBuf,
        provider: CloudProvider,
        state_bucket_name: &str,
    ) -> TerraformRunner {
        TerraformRunner {
            binary_path,
            working_directory_path: working_directory,
            provider,
            state_bucket_name: state_bucket_name.to_string(),
        }
    }
}
