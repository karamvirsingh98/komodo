use anyhow::{anyhow, Context};
use async_timing_util::{get_timelength_in_ms, unix_timestamp_ms};
use async_trait::async_trait;
use monitor_client::{
  api::read::*,
  entities::{
    deployment::ContainerSummary,
    resource::AddFilters,
    server::{
      docker_image::ImageSummary, docker_network::DockerNetwork,
      stats::SystemInformation, Server, ServerActionState,
      ServerListItem, ServerStatus,
    },
    user::User,
    PermissionLevel,
  },
};
use mungos::{
  find::find_collect,
  mongodb::{
    bson::{doc, Document},
    options::FindOptions,
  },
};
use periphery_client::api::{self, GetAccountsResponse};
use resolver_api::{Resolve, ResolveToString};

use crate::{
  db::db_client,
  helpers::{
    cache::server_status_cache, periphery_client,
    resource::StateResource,
  },
  state::{action_states, State},
};

#[async_trait]
impl Resolve<GetServersSummary, User> for State {
  async fn resolve(
    &self,
    GetServersSummary {}: GetServersSummary,
    user: User,
  ) -> anyhow::Result<GetServersSummaryResponse> {
    let servers =
      <State as StateResource<Server>>::list_resources_for_user(
        self,
        Document::new(),
        &user,
      )
      .await?;
    let mut res = GetServersSummaryResponse::default();
    for server in servers {
      res.total += 1;
      match server.info.status {
        ServerStatus::Ok => {
          res.healthy += 1;
        }
        ServerStatus::NotOk => {
          res.unhealthy += 1;
        }
        ServerStatus::Disabled => {
          res.disabled += 1;
        }
      }
    }
    Ok(res)
  }
}

#[async_trait]
impl Resolve<GetPeripheryVersion, User> for State {
  async fn resolve(
    &self,
    req: GetPeripheryVersion,
    user: User,
  ) -> anyhow::Result<GetPeripheryVersionResponse> {
    let _: Server = self
      .get_resource_check_permissions(
        &req.server_id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    let version = server_status_cache()
      .get(&req.server_id)
      .await
      .map(|s| s.version.clone())
      .unwrap_or(String::from("unknown"));
    Ok(GetPeripheryVersionResponse { version })
  }
}

#[async_trait]
impl Resolve<GetServer, User> for State {
  async fn resolve(
    &self,
    req: GetServer,
    user: User,
  ) -> anyhow::Result<Server> {
    self
      .get_resource_check_permissions(
        &req.id,
        &user,
        PermissionLevel::Read,
      )
      .await
  }
}

#[async_trait]
impl Resolve<ListServers, User> for State {
  async fn resolve(
    &self,
    ListServers { query }: ListServers,
    user: User,
  ) -> anyhow::Result<Vec<ServerListItem>> {
    let mut filters = Document::new();
    query.add_filters(&mut filters);
    <State as StateResource<Server>>::list_resources_for_user(
      self, filters, &user,
    )
    .await
  }
}

#[async_trait]
impl Resolve<GetServerStatus, User> for State {
  async fn resolve(
    &self,
    GetServerStatus { id }: GetServerStatus,
    user: User,
  ) -> anyhow::Result<GetServerStatusResponse> {
    let _: Server = self
      .get_resource_check_permissions(
        &id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    let status = server_status_cache()
      .get(&id)
      .await
      .ok_or(anyhow!("did not find cached status for server"))?;
    let response = GetServerStatusResponse {
      status: status.status,
    };
    Ok(response)
  }
}

#[async_trait]
impl Resolve<GetServerActionState, User> for State {
  async fn resolve(
    &self,
    GetServerActionState { id }: GetServerActionState,
    user: User,
  ) -> anyhow::Result<ServerActionState> {
    let _: Server = self
      .get_resource_check_permissions(
        &id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    let action_state =
      action_states().server.get(&id).await.unwrap_or_default();
    Ok(action_state)
  }
}

#[async_trait]
impl Resolve<GetSystemInformation, User> for State {
  async fn resolve(
    &self,
    GetSystemInformation { server_id }: GetSystemInformation,
    user: User,
  ) -> anyhow::Result<SystemInformation> {
    let server: Server = self
      .get_resource_check_permissions(
        &server_id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    periphery_client(&server)?
      .request(api::stats::GetSystemInformation {})
      .await
  }
}

#[async_trait]
impl ResolveToString<GetAllSystemStats, User> for State {
  async fn resolve_to_string(
    &self,
    GetAllSystemStats { server_id }: GetAllSystemStats,
    user: User,
  ) -> anyhow::Result<String> {
    let _: Server = self
      .get_resource_check_permissions(
        &server_id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    let status =
      server_status_cache().get(&server_id).await.with_context(
        || format!("did not find status for server at {server_id}"),
      )?;
    let stats = status
      .stats
      .as_ref()
      .ok_or(anyhow!("server not reachable"))?;
    let stats = serde_json::to_string(&stats)?;
    Ok(stats)
  }
}

#[async_trait]
impl ResolveToString<GetBasicSystemStats, User> for State {
  async fn resolve_to_string(
    &self,
    GetBasicSystemStats { server_id }: GetBasicSystemStats,
    user: User,
  ) -> anyhow::Result<String> {
    let _: Server = self
      .get_resource_check_permissions(
        &server_id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    let status = server_status_cache().get(&server_id).await.ok_or(
      anyhow!("did not find status for server at {server_id}"),
    )?;
    let stats = status
      .stats
      .as_ref()
      .ok_or(anyhow!("server not reachable"))?;
    let stats = serde_json::to_string(&stats.basic)?;
    Ok(stats)
  }
}

#[async_trait]
impl ResolveToString<GetCpuUsage, User> for State {
  async fn resolve_to_string(
    &self,
    GetCpuUsage { server_id }: GetCpuUsage,
    user: User,
  ) -> anyhow::Result<String> {
    let _: Server = self
      .get_resource_check_permissions(
        &server_id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    let status = server_status_cache().get(&server_id).await.ok_or(
      anyhow!("did not find status for server at {server_id}"),
    )?;
    let stats = status
      .stats
      .as_ref()
      .ok_or(anyhow!("server not reachable"))?;
    let stats = serde_json::to_string(&stats.cpu)?;
    Ok(stats)
  }
}

#[async_trait]
impl ResolveToString<GetDiskUsage, User> for State {
  async fn resolve_to_string(
    &self,
    GetDiskUsage { server_id }: GetDiskUsage,
    user: User,
  ) -> anyhow::Result<String> {
    let _: Server = self
      .get_resource_check_permissions(
        &server_id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    let status = server_status_cache().get(&server_id).await.ok_or(
      anyhow!("did not find status for server at {server_id}"),
    )?;
    let stats = status
      .stats
      .as_ref()
      .ok_or(anyhow!("server not reachable"))?;
    let stats = serde_json::to_string(&stats.disk)?;
    Ok(stats)
  }
}

#[async_trait]
impl ResolveToString<GetNetworkUsage, User> for State {
  async fn resolve_to_string(
    &self,
    GetNetworkUsage { server_id }: GetNetworkUsage,
    user: User,
  ) -> anyhow::Result<String> {
    let _: Server = self
      .get_resource_check_permissions(
        &server_id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    let status = server_status_cache().get(&server_id).await.ok_or(
      anyhow!("did not find status for server at {server_id}"),
    )?;
    let stats = status
      .stats
      .as_ref()
      .ok_or(anyhow!("server not reachable"))?;
    let stats = serde_json::to_string(&stats.network)?;
    Ok(stats)
  }
}

#[async_trait]
impl ResolveToString<GetSystemProcesses, User> for State {
  async fn resolve_to_string(
    &self,
    GetSystemProcesses { server_id }: GetSystemProcesses,
    user: User,
  ) -> anyhow::Result<String> {
    let _: Server = self
      .get_resource_check_permissions(
        &server_id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    let status = server_status_cache().get(&server_id).await.ok_or(
      anyhow!("did not find status for server at {server_id}"),
    )?;
    let stats = status
      .stats
      .as_ref()
      .ok_or(anyhow!("server not reachable"))?;
    let stats = serde_json::to_string(&stats.processes)?;
    Ok(stats)
  }
}

#[async_trait]
impl ResolveToString<GetSystemComponents, User> for State {
  async fn resolve_to_string(
    &self,
    GetSystemComponents { server_id }: GetSystemComponents,
    user: User,
  ) -> anyhow::Result<String> {
    let _: Server = self
      .get_resource_check_permissions(
        &server_id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    let status = server_status_cache().get(&server_id).await.ok_or(
      anyhow!("did not find status for server at {server_id}"),
    )?;
    let stats = status
      .stats
      .as_ref()
      .ok_or(anyhow!("server not reachable"))?;
    let stats = serde_json::to_string(&stats.components)?;
    Ok(stats)
  }
}

const STATS_PER_PAGE: i64 = 500;

#[async_trait]
impl Resolve<GetHistoricalServerStats, User> for State {
  async fn resolve(
    &self,
    GetHistoricalServerStats {
      server_id,
      interval,
      page,
    }: GetHistoricalServerStats,
    user: User,
  ) -> anyhow::Result<GetHistoricalServerStatsResponse> {
    let _: Server = self
      .get_resource_check_permissions(
        &server_id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    let interval =
      get_timelength_in_ms(interval.to_string().parse().unwrap())
        as i64;
    let mut ts_vec = Vec::<i64>::new();
    let curr_ts = unix_timestamp_ms() as i64;
    let mut curr_ts = curr_ts
      - curr_ts % interval
      - interval * STATS_PER_PAGE * page as i64;
    for _ in 0..STATS_PER_PAGE {
      ts_vec.push(curr_ts);
      curr_ts -= interval;
    }

    let stats = find_collect(
      &db_client().await.stats,
      doc! {
        "sid": server_id,
        "ts": { "$in": ts_vec },
      },
      FindOptions::builder()
        .sort(doc! { "ts": -1 })
        .skip(page as u64 * STATS_PER_PAGE as u64)
        .limit(STATS_PER_PAGE)
        .build(),
    )
    .await
    .context("failed to pull stats from db")?;
    let next_page = if stats.len() == STATS_PER_PAGE as usize {
      Some(page + 1)
    } else {
      None
    };
    let res = GetHistoricalServerStatsResponse { stats, next_page };
    Ok(res)
  }
}

#[async_trait]
impl Resolve<GetDockerImages, User> for State {
  async fn resolve(
    &self,
    GetDockerImages { server_id }: GetDockerImages,
    user: User,
  ) -> anyhow::Result<Vec<ImageSummary>> {
    let server: Server = self
      .get_resource_check_permissions(
        &server_id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    periphery_client(&server)?
      .request(api::build::GetImageList {})
      .await
  }
}

#[async_trait]
impl Resolve<GetDockerNetworks, User> for State {
  async fn resolve(
    &self,
    GetDockerNetworks { server_id }: GetDockerNetworks,
    user: User,
  ) -> anyhow::Result<Vec<DockerNetwork>> {
    let server: Server = self
      .get_resource_check_permissions(
        &server_id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    periphery_client(&server)?
      .request(api::network::GetNetworkList {})
      .await
  }
}

#[async_trait]
impl Resolve<GetDockerContainers, User> for State {
  async fn resolve(
    &self,
    GetDockerContainers { server_id }: GetDockerContainers,
    user: User,
  ) -> anyhow::Result<Vec<ContainerSummary>> {
    let server: Server = self
      .get_resource_check_permissions(
        &server_id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    periphery_client(&server)?
      .request(api::container::GetContainerList {})
      .await
  }
}

#[async_trait]
impl Resolve<GetAvailableAccounts, User> for State {
  async fn resolve(
    &self,
    GetAvailableAccounts { server_id }: GetAvailableAccounts,
    user: User,
  ) -> anyhow::Result<GetAvailableAccountsResponse> {
    let server: Server = self
      .get_resource_check_permissions(
        &server_id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    let GetAccountsResponse { github, docker } =
      periphery_client(&server)?
        .request(api::GetAccounts {})
        .await
        .context("failed to get accounts from periphery")?;
    let res = GetAvailableAccountsResponse { github, docker };
    Ok(res)
  }
}

#[async_trait]
impl Resolve<GetAvailableSecrets, User> for State {
  async fn resolve(
    &self,
    GetAvailableSecrets { server_id }: GetAvailableSecrets,
    user: User,
  ) -> anyhow::Result<GetAvailableSecretsResponse> {
    let server: Server = self
      .get_resource_check_permissions(
        &server_id,
        &user,
        PermissionLevel::Read,
      )
      .await?;
    let secrets = periphery_client(&server)?
      .request(api::GetSecrets {})
      .await
      .context("failed to get accounts from periphery")?;
    Ok(secrets)
  }
}
