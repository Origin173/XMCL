pub mod background;
pub mod commands;
pub mod download;
pub mod events;
pub mod monitor;
pub mod streams;

use crate::error::XMCLResult;
use download::DownloadParam;
use events::TauriEventSink;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use streams::{GDesc, PDesc, PHandle};
use tokio::time::Duration;

pub type XMCLBoxedFuture = Pin<Box<dyn Future<Output = XMCLResult<()>> + Send>>;

pub struct XMCLFuture {
  pub task_id: u32,
  pub task_group: Option<String>,
  pub f: XMCLBoxedFuture,
}

pub struct XMCLFutureDesc {
  pub task_id: u32,
  pub f: XMCLBoxedFuture,
  pub h: Arc<RwLock<PTaskHandle>>,
}

type PTaskHandle = PHandle<TauriEventSink, PTaskParam>;
type PTaskDesc = PDesc<PTaskParam>;
type PTaskGroupDesc = GDesc<PTaskParam>;

#[derive(Serialize, Deserialize, Clone)]
pub struct THandle {
  #[serde(default)]
  pub task_id: u32,
  pub task_group: Option<String>,
  pub task_type: String,
  pub state: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "taskType", rename_all = "camelCase")]
pub enum PTaskParam {
  Download(DownloadParam),
}
