// Copyright (c) 2023, MaidSafe.
// All rights reserved.
//
// This SAFE Network Software is licensed under the BSD-3-Clause license.
// Please see the LICENSE file for more details.

use super::super::{CloudProvider, TestnetDeploy};
use super::setup::*;
use crate::ansible::MockAnsibleRunnerInterface;
use crate::rpc_client::{MockRpcClientInterface, NodeInfo};
use crate::s3::MockS3RepositoryInterface;
use crate::ssh::MockSshClientInterface;
use crate::terraform::MockTerraformRunnerInterface;
use color_eyre::Result;
use mockall::predicate::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

#[tokio::test]
async fn should_return_the_genesis_multiaddr() -> Result<()> {
    let (tmp_dir, working_dir) = setup_working_directory()?;
    let mut s3_repository = MockS3RepositoryInterface::new();
    s3_repository.expect_download_object().times(0);
    let mut ansible_runner = MockAnsibleRunnerInterface::new();
    ansible_runner
        .expect_inventory_list()
        .times(1)
        .with(eq(
            PathBuf::from("inventory").join(".beta_genesis_inventory_digital_ocean.yml")
        ))
        .returning(|_| {
            Ok(vec![(
                "beta-genesis".to_string(),
                IpAddr::V4(Ipv4Addr::new(10, 0, 0, 10)),
            )])
        });

    let addr: SocketAddr = "10.0.0.10:12001".parse()?;
    let mut rpc_client = MockRpcClientInterface::new();
    rpc_client
        .expect_get_info()
        .times(1)
        .with(eq(addr))
        .returning(|_| Ok(NodeInfo {
            endpoint: "https://10.0.0.1:12001".to_string(),
            peer_id: "12D3KooWLvmkUDQRthtZv9CrzozRLk9ZVEHXgmx6UxVMiho5aded".to_string(),
            logs_dir: PathBuf::from("/home/safe/.local/share/safe/node/12D3KooWLvmkUDQRthtZv9CrzozRLk9ZVEHXgmx6UxVMiho5aded/logs"),
            pid: 4067,
            safenode_version: "0.88.16".to_string(),
            last_restart: 187
        }));

    let testnet = TestnetDeploy::new(
        Box::new(MockTerraformRunnerInterface::new()),
        Box::new(ansible_runner),
        Box::new(rpc_client),
        Box::new(MockSshClientInterface::new()),
        working_dir.to_path_buf(),
        CloudProvider::DigitalOcean,
        Box::new(s3_repository),
    );

    let (multiaddr, genesis_ip) = testnet.get_genesis_multiaddr("beta").await?;

    assert_eq!(
        multiaddr,
        "/ip4/10.0.0.10/tcp/12000/p2p/12D3KooWLvmkUDQRthtZv9CrzozRLk9ZVEHXgmx6UxVMiho5aded"
    );
    assert_eq!(genesis_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 10)));

    drop(tmp_dir);
    Ok(())
}
