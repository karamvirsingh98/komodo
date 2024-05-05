use anyhow::{anyhow, Context};
use futures::future::join_all;
use monitor_client::entities::{
  alert::{Alert, AlertData},
  alerter::*,
  deployment::DockerContainerState,
  server::stats::SeverityLevel,
};
use mungos::{find::find_collect, mongodb::bson::doc};
use reqwest::StatusCode;
use slack::types::Block;

use crate::state::db_client;

#[instrument]
pub async fn send_alerts(alerts: &[Alert]) {
  if alerts.is_empty() {
    return;
  }

  let alerters = find_collect(
    &db_client().await.alerters,
    doc! { "config.params.enabled": true },
    None,
  )
  .await;

  if let Err(e) = alerters {
    error!(
      "ERROR sending alerts | failed to get alerters from db | {e:#}"
    );
    return;
  }

  let alerters = alerters.unwrap();

  let handles =
    alerts.iter().map(|alert| send_alert(&alerters, alert));

  join_all(handles).await;
}

#[instrument(level = "debug")]
async fn send_alert(alerters: &[Alerter], alert: &Alert) {
  if alerters.is_empty() {
    return;
  }

  let handles = alerters.iter().map(|alerter| async {
    match &alerter.config {
      AlerterConfig::Slack(SlackAlerterConfig { url, enabled }) => {
        if !enabled {
          return Ok(());
        }
        send_slack_alert(url, alert)
          .await
          .context("failed to send slack alert")
      }
      AlerterConfig::Custom(CustomAlerterConfig { url, enabled }) => {
        if !enabled {
          return Ok(());
        }
        send_custom_alert(url, alert).await.context(format!(
          "failed to send alert to custom alerter at {url}"
        ))
      }
    }
  });

  join_all(handles)
    .await
    .into_iter()
    .filter_map(|res| res.err())
    .for_each(|e| error!("{e:#}"));
}

#[instrument(level = "debug")]
async fn send_custom_alert(
  url: &str,
  alert: &Alert,
) -> anyhow::Result<()> {
  let res = reqwest::Client::new()
    .post(url)
    .json(alert)
    .send()
    .await
    .context("failed at post request to alerter")?;
  let status = res.status();
  if status != StatusCode::OK {
    let text = res
      .text()
      .await
      .context("failed to get response text on alerter response")?;
    return Err(anyhow!(
      "post to alerter failed | {status} | {text}"
    ));
  }
  Ok(())
}

#[instrument(level = "debug")]
async fn send_slack_alert(
  url: &str,
  alert: &Alert,
) -> anyhow::Result<()> {
  let level = fmt_level(alert.level);
  let (text, blocks): (_, Option<_>) = match &alert.data {
    AlertData::ServerUnreachable { name, region, .. } => {
      let region = fmt_region(region);
      match alert.level {
        SeverityLevel::Ok => {
          let text =
            format!("{level} | *{name}*{region} is now *reachable*");
          let blocks = vec![
            Block::header(level),
            Block::section(format!(
              "*{name}*{region} is now *reachable*"
            )),
          ];
          (text, blocks.into())
        }
        SeverityLevel::Critical => {
          let text =
            format!("{level} | *{name}*{region} is *unreachable* ❌");
          let blocks = vec![
            Block::header(level),
            Block::section(format!(
              "*{name}*{region} is *unreachable* ❌"
            )),
          ];
          (text, blocks.into())
        }
        _ => unreachable!(),
      }
    }
    AlertData::ServerCpu {
      name,
      region,
      percentage,
      ..
    } => {
      let region = fmt_region(region);
      let text = format!("{level} | *{name}*{region} cpu usage at *{percentage:.1}%* 📈 🚨");
      let blocks = vec![
        Block::header(format!("{level} 🚨")),
        Block::section(format!(
          "*{name}*{region} cpu usage at *{percentage:.1}%* 📈 🚨"
        )),
      ];
      (text, blocks.into())
    }
    AlertData::ServerMem {
      name,
      region,
      used_gb,
      total_gb,
      ..
    } => {
      let region = fmt_region(region);
      let percentage = 100.0 * used_gb / total_gb;
      let text =
                format!("{level} | *{name}*{region} memory usage at *{percentage:.1}%* 💾 🚨");
      let blocks = vec![
        Block::header(level),
        Block::section(format!(
          "*{name}*{region} memory usage at *{percentage:.1}%* 💾 🚨"
        )),
        Block::section(format!(
          "using *{used_gb:.1} GiB* / *{total_gb:.1} GiB*"
        )),
      ];
      (text, blocks.into())
    }
    AlertData::ServerDisk {
      name,
      region,
      path,
      used_gb,
      total_gb,
      ..
    } => {
      let region = fmt_region(region);
      let percentage = 100.0 * used_gb / total_gb;
      let text = format!("{level} | *{name}*{region} disk usage at *{percentage:.1}%* | mount point: *{path:?}* 💿 🚨");
      let blocks = vec![
        Block::header(level),
        Block::section(format!(
          "*{name}*{region} disk usage at *{percentage:.1}%* 💿 🚨"
        )),
        Block::section(format!(
          "mount point: {path:?} | using *{used_gb:.1} GiB* / *{total_gb:.1} GiB*"
        )),
      ];
      (text, blocks.into())
    }
    AlertData::ContainerStateChange {
      name,
      server_name,
      from,
      to,
      ..
    } => {
      let to = fmt_docker_container_state(to);
      let text = format!("📦 container *{name}* is now {to}");
      let blocks = vec![
        Block::header(text.clone()),
        Block::section(format!(
          "server: {server_name}\nprevious: {from}"
        )),
      ];
      (text, blocks.into())
    }
    AlertData::AwsBuilderTerminationFailed { instance_id } => {
      let text = format!(
        "{level} | Failed to terminated AWS builder instance"
      );
      let blocks = vec![
        Block::header(text.clone()),
        Block::section(format!("instance id: {instance_id}")),
      ];
      (text, blocks.into())
    }
    AlertData::None {} => Default::default(),
  };
  if !text.is_empty() {
    let slack = slack::Client::new(url);
    slack.send_message(text, blocks).await?;
  }
  Ok(())
}

fn fmt_region(region: &Option<String>) -> String {
  match region {
    Some(region) => format!(" ({region})"),
    None => String::new(),
  }
}

fn fmt_docker_container_state(
  state: &DockerContainerState,
) -> String {
  match state {
    DockerContainerState::Running => String::from("Running ▶️"),
    DockerContainerState::Exited => String::from("Exited 🛑"),
    DockerContainerState::Restarting => String::from("Restarting 🔄"),
    DockerContainerState::NotDeployed => String::from("Not Deployed"),
    _ => state.to_string(),
  }
}

fn fmt_level(level: SeverityLevel) -> &'static str {
  match level {
    SeverityLevel::Critical => "CRITICAL 🚨",
    SeverityLevel::Warning => "WARNING 🚨",
    SeverityLevel::Ok => "OK ✅",
  }
}