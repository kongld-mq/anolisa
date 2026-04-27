// Transitional re-export layer — all btrfs CLI operations have moved to
// `backends::btrfs_common`.  This module preserves the old import paths during
// the migration period and will be removed in a future PR.
pub use crate::backends::btrfs_common::*;
