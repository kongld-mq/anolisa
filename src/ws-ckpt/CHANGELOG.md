# Changelog

## 0.1.0

- Daemon with Unix Socket IPC and Bincode binary protocol.
- `init` / `checkpoint` / `rollback` / `delete` / `list` / `diff` / `cleanup` / `status` / `config` commands.
- Background scheduler: auto-cleanup, health check, orphan recovery.
- Multi-backend: btrfs-base / btrfs-loop / overlayfs with auto-detection.
- TOML config persistence with runtime hot-reload.
- systemd service with RPM packaging for Alinux 4.
