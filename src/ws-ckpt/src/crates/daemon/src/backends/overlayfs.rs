use std::path::{Path, PathBuf};

use async_trait::async_trait;

use ws_ckpt_common::backend::*;
use ws_ckpt_common::{DiffEntry, WorkspaceInfo, SNAPSHOTS_DIR};

pub struct OverlayFsBackend {
    data_root: PathBuf,
    snapshots_dir: PathBuf,
}

impl OverlayFsBackend {
    pub fn new(data_root: PathBuf) -> Self {
        let snapshots_dir = data_root.join(SNAPSHOTS_DIR);
        Self {
            data_root,
            snapshots_dir,
        }
    }
}

#[async_trait]
impl StorageBackend for OverlayFsBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::OverlayFs
    }

    fn data_root(&self) -> &Path {
        &self.data_root
    }

    fn snapshots_root(&self) -> &Path {
        &self.snapshots_dir
    }

    async fn init_workspace(
        &self,
        _original_path: &str,
        _ws_id: &str,
    ) -> anyhow::Result<WorkspaceInfo> {
        todo!("OverlayFs backend: init_workspace not implemented yet")
    }

    async fn create_snapshot(&self, _ws_id: &str, _snapshot_id: &str) -> anyhow::Result<()> {
        todo!("OverlayFs backend: create_snapshot not implemented yet")
    }

    async fn rollback(&self, _ws_id: &str, _snapshot_id: &str) -> anyhow::Result<PathBuf> {
        todo!("OverlayFs backend: rollback not implemented yet")
    }

    async fn delete_snapshot(&self, _ws_id: &str, _snapshot_id: &str) -> anyhow::Result<()> {
        todo!("OverlayFs backend: delete_snapshot not implemented yet")
    }

    async fn recover_workspace(&self, _ws_id: &str, _original_path: &str) -> anyhow::Result<()> {
        todo!("OverlayFs backend: recover_workspace not implemented yet")
    }

    async fn diff(&self, _ws_id: &str, _from: &str, _to: &str) -> anyhow::Result<Vec<DiffEntry>> {
        todo!("OverlayFs backend: diff not implemented yet")
    }

    async fn cleanup_snapshots(
        &self,
        _ws_id: &str,
        _snapshot_ids: &[String],
    ) -> anyhow::Result<Vec<String>> {
        todo!("OverlayFs backend: cleanup_snapshots not implemented yet")
    }

    async fn fork(&self, _ws_id: &str, _snapshot_id: &str, _new_ws_id: &str) -> anyhow::Result<()> {
        todo!("OverlayFs backend: fork not implemented yet")
    }

    async fn gc_generations(&self, _ws_id: &str) -> anyhow::Result<GcResult> {
        todo!("OverlayFs backend: gc_generations not implemented yet")
    }

    async fn check_environment(&self) -> anyhow::Result<EnvironmentStatus> {
        todo!("OverlayFs backend: check_environment not implemented yet")
    }

    async fn get_usage(&self) -> anyhow::Result<(u64, u64)> {
        todo!("OverlayFs backend: get_usage not implemented yet")
    }
}
