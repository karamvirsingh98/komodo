use anyhow::Context;
use monitor_types::{Deployment, DeploymentActionState, DeploymentWithContainer, Update};
use serde_json::{json, Value};

use crate::MonitorClient;

impl MonitorClient {
    pub async fn list_deployments(
        &self,
        query: impl Into<Option<Value>>,
    ) -> anyhow::Result<Vec<DeploymentWithContainer>> {
        self.get("/api/deployment/list", query.into())
            .await
            .context("failed at list deployments")
    }

    pub async fn get_deployment(
        &self,
        deployment_id: &str,
    ) -> anyhow::Result<DeploymentWithContainer> {
        self.get(
            &format!("/api/deployment/{deployment_id}"),
            Option::<()>::None,
        )
        .await
        .context(format!("failed at get deployment {deployment_id}"))
    }

    pub async fn get_deployment_action_state(
        &self,
        deployment_id: &str,
    ) -> anyhow::Result<DeploymentActionState> {
        self.get(
            &format!("/api/deployment/{deployment_id}/action_state"),
            Option::<()>::None,
        )
        .await
    }

    pub async fn create_deployment(
        &self,
        name: &str,
        server_id: &str,
    ) -> anyhow::Result<Deployment> {
        self.post(
            "/api/deployment/create",
            json!({ "name": name, "server_id": server_id }),
        )
        .await
        .context(format!(
            "failed at create deployment with name {name} and server id {server_id}"
        ))
    }

    pub async fn create_full_deployment(
        &self,
        deployment: &Deployment,
    ) -> anyhow::Result<Deployment> {
        self.post("/api/deployment/create_full", deployment)
            .await
            .context(format!("failed at creating full deployment"))
    }

    pub async fn delete_deployment(&self, id: &str) -> anyhow::Result<Deployment> {
        self.delete::<(), _>(&format!("/api/deployment/{id}/delete"), None)
            .await
            .context(format!("failed at deleting deployment {id}"))
    }

    pub async fn update_deployment(&self, deployment: Deployment) -> anyhow::Result<Deployment> {
        self.patch("/api/deployment/update", deployment)
            .await
            .context("failed at updating deployment")
    }

    pub async fn reclone_deployment(&self, id: &str) -> anyhow::Result<Update> {
        self.post::<(), _>(&format!("/api/deployment/{id}/reclone"), None)
            .await
            .context(format!("failed at reclone deployment {id}"))
    }

    pub async fn deploy_container(&self, deployment_id: &str) -> anyhow::Result<Update> {
        self.post::<(), _>(&format!("/api/deployment/{deployment_id}/deploy"), None)
            .await
            .context(format!("failed at deploy deployment {deployment_id}"))
    }

    pub async fn start_container(&self, deployment_id: &str) -> anyhow::Result<Update> {
        self.post::<(), _>(
            &format!("/api/deployment/{deployment_id}/start_container"),
            None,
        )
        .await
        .context(format!(
            "failed at start container for deployment {deployment_id}"
        ))
    }

    pub async fn stop_container(&self, deployment_id: &str) -> anyhow::Result<Update> {
        self.post::<(), _>(
            &format!("/api/deployment/{deployment_id}/stop_container"),
            None,
        )
        .await
        .context(format!(
            "failed at stop container for deployment {deployment_id}"
        ))
    }

    pub async fn remove_container(&self, deployment_id: &str) -> anyhow::Result<Update> {
        self.post::<(), _>(
            &format!("/api/deployment/{deployment_id}/remove_container"),
            None,
        )
        .await
        .context(format!(
            "failed at remove container for deployment {deployment_id}"
        ))
    }

    pub async fn pull_deployment_repo(&self, deployment_id: &str) -> anyhow::Result<Update> {
        self.post::<(), _>(&format!("/api/deployment/{deployment_id}/pull"), None)
            .await
            .context(format!(
                "failed at remove container for deployment {deployment_id}"
            ))
    }
}
