use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 后端类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendType {
    BtrfsLoop, // 回环设备上的 btrfs（当前实现）
    BtrfsBase, // 原生 btrfs 分区/子卷
    OverlayFs, // OverlayFS + XFS reflink（预留）
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendType::BtrfsLoop => write!(f, "btrfs-loop"),
            BackendType::BtrfsBase => write!(f, "btrfs-base"),
            BackendType::OverlayFs => write!(f, "overlayfs"),
        }
    }
}

/// 清理结果
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CleanupResult {
    pub removed: Vec<String>, // 被清理的快照 ID 列表
    pub kept: usize,          // 保留的快照数
}

/// GC 结果（overlayfs generation 清理，btrfs 后端空实现）
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GcResult {
    pub generations_removed: usize,
}

/// 环境检查状态
#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentStatus {
    pub backend: BackendType,
    pub healthy: bool,
    pub details: Vec<String>, // 检查项描述
}

/// StorageBackend trait — 所有存储后端必须实现
///
/// 编排层（dispatcher/workspace_mgr/snapshot_mgr）通过此 trait 调用存储操作。
/// WS ID 生成、index.json 管理、daemon state 注册等逻辑不在 trait 中，由编排层负责。
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// 返回后端类型标识
    fn backend_type(&self) -> BackendType;

    /// 返回后端数据根目录（工作区子卷和快照的父目录）
    fn data_root(&self) -> &std::path::Path;

    /// 返回快照存储根目录
    fn snapshots_root(&self) -> &std::path::Path;

    /// 初始化工作区
    /// - ws_id 由编排层生成（ws-{SHA256(path)[:6]}）
    /// - btrfs-base: 情景A mv + subvol + symlink / 情景B rsync + subvol + symlink
    /// - btrfs-loop: rsync + 创建 img + mkfs + losetup + mount + subvol + symlink
    async fn init_workspace(
        &self,
        original_path: &str,
        ws_id: &str,
    ) -> anyhow::Result<crate::WorkspaceInfo>;

    /// 创建快照
    /// - btrfs: btrfs subvolume snapshot -r
    async fn create_snapshot(&self, ws_id: &str, snapshot_id: &str) -> anyhow::Result<()>;

    /// 回滚到指定快照
    /// - 所有 btrfs 后端：创建可写子卷 + symlink 原子切换（ln -s + mv -T）
    async fn rollback(&self, ws_id: &str, snapshot_id: &str) -> anyhow::Result<PathBuf>;

    /// 删除快照子卷
    async fn delete_snapshot(&self, ws_id: &str, snapshot_id: &str) -> anyhow::Result<()>;

    /// 恢复工作区为普通目录（撤销 init）
    /// - btrfs-base: rsync 还原 + 删 symlink + 删子卷（无 umount loop）
    /// - btrfs-loop: rsync 还原 + 删 symlink + 删子卷 + umount + losetup -d + 删 img
    async fn recover_workspace(&self, ws_id: &str, original_path: &str) -> anyhow::Result<()>;

    /// 获取两个快照之间的 diff
    async fn diff(
        &self,
        ws_id: &str,
        from: &str,
        to: &str,
    ) -> anyhow::Result<Vec<crate::DiffEntry>>;

    /// 清理旧快照（保留最近 keep 个 + 所有 pinned）
    /// 返回被删除的快照 ID 列表
    async fn cleanup_snapshots(
        &self,
        ws_id: &str,
        snapshot_ids: &[String],
    ) -> anyhow::Result<Vec<String>>;

    /// 从快照 fork 出独立工作区（overlayfs 预留接口）
    async fn fork(&self, ws_id: &str, snapshot_id: &str, new_ws_id: &str) -> anyhow::Result<()>;

    /// 清理旧 generation（overlayfs 预留接口，btrfs 后端空实现）
    async fn gc_generations(&self, ws_id: &str) -> anyhow::Result<GcResult>;

    /// 环境检查
    async fn check_environment(&self) -> anyhow::Result<EnvironmentStatus>;

    /// 获取文件系统使用率（总计, 已用）字节
    async fn get_usage(&self) -> anyhow::Result<(u64, u64)>;
}
