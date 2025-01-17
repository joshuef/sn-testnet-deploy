// Copyright (c) 2023, MaidSafe.
// All rights reserved.
//
// This SAFE Network Software is licensed under the BSD-3-Clause license.
// Please see the LICENSE file for more details.

use super::super::{CloudProvider, TestnetDeploy};
use super::setup::*;
use crate::ansible::MockAnsibleRunnerInterface;
use crate::rpc_client::MockRpcClientInterface;
use crate::ssh::MockSshClientInterface;
use crate::terraform::MockTerraformRunnerInterface;
use assert_fs::prelude::*;
use color_eyre::{eyre::eyre, Result};
use mockall::predicate::*;
use mockall::Sequence;

#[tokio::test]
async fn should_run_terraform_destroy_and_delete_workspace_and_delete_inventory_files() -> Result<()>
{
    let (tmp_dir, working_dir) = setup_working_directory()?;
    let s3_repository = setup_default_s3_repository(&working_dir)?;
    let mut terraform_runner = setup_default_terraform_runner("alpha");
    let mut seq = Sequence::new();
    terraform_runner
        .expect_workspace_list()
        .times(1)
        .returning(|| {
            Ok(vec![
                "alpha".to_string(),
                "default".to_string(),
                "dev".to_string(),
            ])
        });
    terraform_runner
        .expect_workspace_select()
        .times(1)
        .in_sequence(&mut seq)
        .with(eq("alpha".to_string()))
        .returning(|_| Ok(()));
    terraform_runner
        .expect_destroy()
        .times(1)
        .returning(|| Ok(()));
    terraform_runner
        .expect_workspace_select()
        .times(1)
        .in_sequence(&mut seq)
        .with(eq("dev".to_string()))
        .returning(|_| Ok(()));
    terraform_runner
        .expect_workspace_delete()
        .times(1)
        .with(eq("alpha".to_string()))
        .returning(|_| Ok(()));

    let testnet = TestnetDeploy::new(
        Box::new(terraform_runner),
        Box::new(MockAnsibleRunnerInterface::new()),
        Box::new(MockRpcClientInterface::new()),
        Box::new(MockSshClientInterface::new()),
        working_dir.to_path_buf(),
        CloudProvider::DigitalOcean,
        s3_repository,
    );

    // Calling init will create the Ansible inventory files, which we want to be removed by
    // the clean operation.
    testnet.init("alpha").await?;
    testnet.clean("alpha").await?;

    let inventory_types = ["build", "genesis", "node"];
    for inventory_type in inventory_types.iter() {
        let inventory_file = working_dir.child(format!(
            "ansible/inventory/.{}_{}_inventory_digital_ocean.yml",
            "alpha", inventory_type
        ));
        inventory_file.assert(predicates::path::missing());
    }

    drop(tmp_dir);
    Ok(())
}

#[tokio::test]
async fn should_return_an_error_when_invalid_name_is_supplied() -> Result<()> {
    let (tmp_dir, working_dir) = setup_working_directory()?;
    let s3_repository = setup_default_s3_repository(&working_dir)?;
    let mut terraform_runner = MockTerraformRunnerInterface::new();
    terraform_runner
        .expect_workspace_list()
        .times(1)
        .returning(|| {
            Ok(vec![
                "alpha".to_string(),
                "default".to_string(),
                "dev".to_string(),
            ])
        });

    let testnet = TestnetDeploy::new(
        Box::new(terraform_runner),
        Box::new(MockAnsibleRunnerInterface::new()),
        Box::new(MockRpcClientInterface::new()),
        Box::new(MockSshClientInterface::new()),
        working_dir.to_path_buf(),
        CloudProvider::DigitalOcean,
        s3_repository,
    );

    let result = testnet.clean("beta").await;
    match result {
        Ok(()) => {
            drop(tmp_dir);
            Err(eyre!("deploy should have returned an error"))
        }
        Err(e) => {
            assert_eq!(e.to_string(), "The 'beta' environment does not exist");
            drop(tmp_dir);
            Ok(())
        }
    }
}
