use anyhow::{anyhow, Context};
use diff::Diff;
use helpers::to_monitor_name;
use types::{
    monitor_timestamp,
    traits::{Busy, Permissioned},
    Log, Operation, PermissionLevel, Server, Update, UpdateStatus, UpdateTarget,
};

use crate::{auth::RequestUser, state::State};

impl State {
    pub async fn get_server_check_permissions(
        &self,
        server_id: &str,
        user: &RequestUser,
        permission_level: PermissionLevel,
    ) -> anyhow::Result<Server> {
        let server = self.db.get_server(server_id).await?;
        let permissions = server.get_user_permissions(&user.id);
        if user.is_admin || permissions >= permission_level {
            Ok(server)
        } else {
            Err(anyhow!(
                "user does not have required permissions on this server"
            ))
        }
    }

    pub fn server_busy(&self, id: &str) -> bool {
        match self.server_action_states.lock().unwrap().get(id) {
            Some(a) => a.busy(),
            None => false,
        }
    }

    pub async fn create_server(
        &self,
        name: &str,
        address: String,
        user: &RequestUser,
    ) -> anyhow::Result<Server> {
        if !user.is_admin && !user.create_server_permissions {
            return Err(anyhow!(
                "user does not have permissions to add server (not admin)"
            ));
        }
        let start_ts = monitor_timestamp();
        let server = Server {
            name: to_monitor_name(name),
            address,
            permissions: [(user.id.clone(), PermissionLevel::Update)]
                .into_iter()
                .collect(),
            created_at: start_ts.clone(),
            updated_at: start_ts.clone(),
            ..Default::default()
        };
        let server_id = self
            .db
            .servers
            .create_one(server)
            .await
            .context("failed to add server to db")?;
        let server = self.db.get_server(&server_id).await?;
        let update = Update {
            target: UpdateTarget::Server(server_id),
            operation: Operation::CreateServer,
            start_ts,
            end_ts: Some(monitor_timestamp()),
            operator: user.id.clone(),
            success: true,
            ..Default::default()
        };
        self.add_update(update).await?;

        Ok(server)
    }

    pub async fn create_full_server(
        &self,
        mut server: Server,
        user: &RequestUser,
    ) -> anyhow::Result<Server> {
        server.id = self
            .create_server(&server.name, server.address.clone(), user)
            .await?
            .id;
        let server = self.update_server(server, user).await?;
        Ok(server)
    }

    pub async fn delete_server(
        &self,
        server_id: &str,
        user: &RequestUser,
    ) -> anyhow::Result<Server> {
        if self.server_busy(server_id) {
            return Err(anyhow!("server busy"));
        }
        let server = self
            .get_server_check_permissions(server_id, user, PermissionLevel::Update)
            .await?;
        let start_ts = monitor_timestamp();
        self.db.servers.delete_one(&server_id).await?;
        let update = Update {
            target: UpdateTarget::System,
            operation: Operation::DeleteServer,
            start_ts,
            end_ts: Some(monitor_timestamp()),
            operator: user.id.clone(),
            logs: vec![Log::simple(
                "delete server",
                format!("deleted server {}", server.name),
            )],
            success: true,
            ..Default::default()
        };
        self.add_update(update).await?;
        Ok(server)
    }

    pub async fn update_server(
        &self,
        mut new_server: Server,
        user: &RequestUser,
    ) -> anyhow::Result<Server> {
        if self.server_busy(&new_server.id) {
            return Err(anyhow!("server busy"));
        }
        let current_server = self
            .get_server_check_permissions(&new_server.id, user, PermissionLevel::Update)
            .await?;
        let start_ts = monitor_timestamp();

        new_server.permissions = current_server.permissions.clone();
        new_server.created_at = current_server.created_at.clone();
        new_server.updated_at = start_ts.clone();

        let diff = current_server.diff(&new_server);

        self.db
            .servers
            .update_one(&new_server.id, mungos::Update::Regular(new_server.clone()))
            .await
            .context("failed at update one server")?;

        let update = Update {
            operation: Operation::UpdateServer,
            target: UpdateTarget::Server(new_server.id.clone()),
            start_ts,
            end_ts: Some(monitor_timestamp()),
            status: UpdateStatus::Complete,
            logs: vec![Log::simple(
                "server update",
                serde_json::to_string_pretty(&diff).unwrap(),
            )],
            operator: user.id.clone(),
            success: true,
            ..Default::default()
        };

        self.add_update(update).await?;
        Ok(new_server)
    }

    pub async fn prune_networks(
        &self,
        server_id: &str,
        user: &RequestUser,
    ) -> anyhow::Result<Update> {
        if self.server_busy(server_id) {
            return Err(anyhow!("server busy"));
        }
        {
            let mut lock = self.server_action_states.lock().unwrap();
            let entry = lock.entry(server_id.to_string()).or_default();
            entry.pruning_networks = true;
        }
        let res = self.prune_networks_inner(server_id, user).await;
        {
            let mut lock = self.server_action_states.lock().unwrap();
            let entry = lock.entry(server_id.to_string()).or_default();
            entry.pruning_networks = false;
        }
        res
    }

    async fn prune_networks_inner(
        &self,
        server_id: &str,
        user: &RequestUser,
    ) -> anyhow::Result<Update> {
        let server = self
            .get_server_check_permissions(server_id, user, PermissionLevel::Execute)
            .await?;

        let start_ts = monitor_timestamp();
        let mut update = Update {
            target: UpdateTarget::Server(server_id.to_owned()),
            operation: Operation::PruneNetworksServer,
            start_ts,
            status: UpdateStatus::InProgress,
            success: true,
            operator: user.id.clone(),
            ..Default::default()
        };
        update.id = self.add_update(update.clone()).await?;

        let log = match self.periphery.network_prune(&server).await.context(format!(
            "failed to prune networks on server {}",
            server.name
        )) {
            Ok(log) => log,
            Err(e) => Log::error("prune networks", format!("{e:#?}")),
        };

        update.success = log.success;
        update.status = UpdateStatus::Complete;
        update.end_ts = Some(monitor_timestamp());
        update.logs.push(log);

        self.update_update(update.clone()).await?;

        Ok(update)
    }

    pub async fn prune_images(
        &self,
        server_id: &str,
        user: &RequestUser,
    ) -> anyhow::Result<Update> {
        if self.server_busy(server_id) {
            return Err(anyhow!("server busy"));
        }
        {
            let mut lock = self.server_action_states.lock().unwrap();
            let entry = lock.entry(server_id.to_string()).or_default();
            entry.pruning_images = true;
        }
        let res = self.prune_images_inner(server_id, user).await;
        {
            let mut lock = self.server_action_states.lock().unwrap();
            let entry = lock.entry(server_id.to_string()).or_default();
            entry.pruning_images = false;
        }
        res
    }

    async fn prune_images_inner(
        &self,
        server_id: &str,
        user: &RequestUser,
    ) -> anyhow::Result<Update> {
        let server = self
            .get_server_check_permissions(server_id, user, PermissionLevel::Execute)
            .await?;
        let start_ts = monitor_timestamp();
        let mut update = Update {
            target: UpdateTarget::Server(server_id.to_owned()),
            operation: Operation::PruneImagesServer,
            start_ts,
            status: UpdateStatus::InProgress,
            success: true,
            operator: user.id.clone(),
            ..Default::default()
        };
        update.id = self.add_update(update.clone()).await?;

        let log = match self
            .periphery
            .image_prune(&server)
            .await
            .context(format!("failed to prune images on server {}", server.name))
        {
            Ok(log) => log,
            Err(e) => Log::error("prune images", format!("{e:#?}")),
        };

        update.success = log.success;
        update.status = UpdateStatus::Complete;
        update.end_ts = Some(monitor_timestamp());
        update.logs.push(log);

        self.update_update(update.clone()).await?;

        Ok(update)
    }

    pub async fn prune_containers(
        &self,
        server_id: &str,
        user: &RequestUser,
    ) -> anyhow::Result<Update> {
        if self.server_busy(server_id) {
            return Err(anyhow!("server busy"));
        }
        {
            let mut lock = self.server_action_states.lock().unwrap();
            let entry = lock.entry(server_id.to_string()).or_default();
            entry.pruning_containers = true;
        }
        let res = self.prune_containers_inner(server_id, user).await;
        {
            let mut lock = self.server_action_states.lock().unwrap();
            let entry = lock.entry(server_id.to_string()).or_default();
            entry.pruning_containers = false;
        }
        res
    }

    async fn prune_containers_inner(
        &self,
        server_id: &str,
        user: &RequestUser,
    ) -> anyhow::Result<Update> {
        let server = self
            .get_server_check_permissions(server_id, user, PermissionLevel::Execute)
            .await?;

        let start_ts = monitor_timestamp();
        let mut update = Update {
            target: UpdateTarget::Server(server_id.to_owned()),
            operation: Operation::PruneContainersServer,
            start_ts,
            status: UpdateStatus::InProgress,
            success: true,
            operator: user.id.clone(),
            ..Default::default()
        };
        update.id = self.add_update(update.clone()).await?;

        let log = match self
            .periphery
            .container_prune(&server)
            .await
            .context(format!(
                "failed to prune containers on server {}",
                server.name
            )) {
            Ok(log) => log,
            Err(e) => Log::error("prune containers", format!("{e:#?}")),
        };

        update.success = log.success;
        update.status = UpdateStatus::Complete;
        update.end_ts = Some(monitor_timestamp());
        update.logs.push(log);

        self.update_update(update.clone()).await?;

        Ok(update)
    }
}
