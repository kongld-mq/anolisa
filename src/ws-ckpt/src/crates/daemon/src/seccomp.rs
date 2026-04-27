//! Seccomp-bpf syscall filtering for the ws-ckpt daemon.
//!
//! Applies a **deny-list** filter: all syscalls are allowed by default,
//! and a curated list of dangerous / irrelevant syscalls is blocked with EPERM.
//!
//! This approach is chosen because the daemon shells out to external commands
//! (btrfs, mount, rsync, systemctl) whose full syscall set is impractical to
//! enumerate in an allow-list.

use std::collections::BTreeMap;

use seccompiler::{
    apply_filter, BpfProgram, SeccompAction, SeccompCmpArgLen, SeccompCmpOp, SeccompCondition,
    SeccompFilter, SeccompRule, TargetArch,
};

/// Blocked syscalls — operations the daemon should never perform.
///
/// Categories:
/// - Kernel/module manipulation
/// - System state changes (reboot, hostname, clock)
/// - Debugging / tracing
/// - Swap management
/// - Accounting / profiling
/// - Key management
/// - Network-related (the daemon only uses Unix domain sockets)
/// - Virtualization
fn blocked_syscalls() -> Vec<i64> {
    vec![
        // ── Kernel module loading ──
        libc::SYS_init_module,
        libc::SYS_finit_module,
        libc::SYS_delete_module,
        // ── System state ──
        libc::SYS_reboot,
        libc::SYS_kexec_load,
        libc::SYS_kexec_file_load,
        libc::SYS_sethostname,
        libc::SYS_setdomainname,
        libc::SYS_settimeofday,
        libc::SYS_adjtimex,
        libc::SYS_clock_adjtime,
        libc::SYS_pivot_root,
        // ── Debugging / tracing ──
        libc::SYS_ptrace,
        libc::SYS_process_vm_readv,
        libc::SYS_process_vm_writev,
        libc::SYS_kcmp,
        // ── Swap management ──
        libc::SYS_swapon,
        libc::SYS_swapoff,
        // ── Accounting / profiling ──
        libc::SYS_acct,
        libc::SYS_perf_event_open,
        // ── Key management ──
        libc::SYS_add_key,
        libc::SYS_request_key,
        libc::SYS_keyctl,
        // ── Virtualization / namespaces (daemon must not create new namespaces) ──
        libc::SYS_unshare,
        libc::SYS_setns,
        // ── Misc dangerous ──
        libc::SYS_lookup_dcookie,
        libc::SYS_bpf, // prevent loading arbitrary BPF programs
        libc::SYS_userfaultfd,
        libc::SYS_move_pages,
    ]
}

/// Create a rule that matches unconditionally (arg0 as u64 >= 0 is always true).
fn unconditional_rule() -> Result<SeccompRule, seccompiler::Error> {
    Ok(SeccompRule::new(vec![SeccompCondition::new(
        0,
        SeccompCmpArgLen::Qword,
        SeccompCmpOp::Ge,
        0,
    )?])?)
}

/// Apply the seccomp deny-list filter to the current thread (and future children).
///
/// Must be called **after** bootstrap (image creation, loop mount) completes,
/// but **before** the listener loop starts.
pub fn apply_seccomp_filter() -> anyhow::Result<()> {
    let mut rules: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();

    for syscall in blocked_syscalls() {
        rules.insert(syscall, vec![unconditional_rule()?]);
    }

    let filter = SeccompFilter::new(
        rules,
        SeccompAction::Allow,                     // default: allow everything
        SeccompAction::Errno(libc::EPERM as u32), // blocked: return EPERM
        TargetArch::x86_64,
    )?;

    let bpf: BpfProgram = filter.try_into()?;
    apply_filter(&bpf)?;

    tracing::info!(
        "Seccomp filter applied: {} dangerous syscalls blocked",
        blocked_syscalls().len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_syscalls_not_empty() {
        assert!(!blocked_syscalls().is_empty());
    }

    #[test]
    fn blocked_syscalls_no_duplicates() {
        let list = blocked_syscalls();
        let mut sorted = list.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(list.len(), sorted.len(), "duplicate syscall numbers found");
    }

    #[test]
    fn filter_builds_successfully() {
        let mut rules: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();
        for syscall in blocked_syscalls() {
            rules.insert(syscall, vec![unconditional_rule().unwrap()]);
        }
        let filter = SeccompFilter::new(
            rules,
            SeccompAction::Allow,
            SeccompAction::Errno(libc::EPERM as u32),
            TargetArch::x86_64,
        );
        assert!(filter.is_ok(), "seccomp filter should build without errors");
    }
}
