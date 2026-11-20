use crate::model::{PetStatus, SystemStats};
use std::time::Instant;
use sysinfo::{Disks, Networks, System};

/// 网络速率归一的带宽基准：10 MB/s 视为 100% 压力。
const NET_BASELINE_BPS: f64 = 10.0 * 1024.0 * 1024.0;
/// 状态切换需连续确认的次数（迟滞，防抖）。
const CONFIRM_TIMES: u8 = 2;

#[cfg(target_os = "windows")]
fn root_mount() -> &'static str {
    "C:\\"
}
#[cfg(target_os = "macos")]
fn root_mount() -> &'static str {
    "/"
}
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn root_mount() -> &'static str {
    "/"
}

pub struct Monitor {
    sys: System,
    disks: Disks,
    networks: Networks,
    prev_rx: u64,
    prev_tx: u64,
    prev_at: Instant,
    pending: PetStatus,
    pending_count: u8,
    current: PetStatus,
}

impl Monitor {
    pub fn new() -> Self {
        let mut sys = System::new();
        // 预热 CPU：sysinfo 需要两次间隔采样才能得到首个使用率。
        sys.refresh_cpu_usage();
        std::thread::sleep(std::time::Duration::from_millis(120));
        sys.refresh_cpu_usage();

        Self {
            sys,
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            prev_rx: 0,
            prev_tx: 0,
            prev_at: Instant::now(),
            pending: PetStatus::Idle,
            pending_count: 0,
            current: PetStatus::Idle,
        }
    }

    pub fn sample(&mut self) -> SystemStats {
        // CPU / 内存
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        let cpu_usage = self.sys.global_cpu_usage();
        let mem_used = self.sys.used_memory();
        let mem_total = self.sys.total_memory();
        let mem_usage = ratio(mem_used, mem_total);

        // 磁盘：只刷系统主盘
        let (disk_used, disk_total) = self.sample_disk();
        let disk_usage = ratio(disk_used, disk_total);

        // 网络：累计值做差值 → 速率
        let (net_rx, net_tx) = self.sample_network();

        let pressure = cpu_usage
            .max(mem_usage)
            .max(disk_usage)
            .max(net_pressure(net_rx.max(net_tx)));
        let status = self.resolve_status(status_from_pressure(pressure));

        SystemStats {
            cpu_usage: round2(cpu_usage),
            mem_used,
            mem_total,
            mem_usage: round2(mem_usage),
            disk_used,
            disk_total,
            disk_usage: round2(disk_usage),
            net_rx,
            net_tx,
            sampled_at: chrono::Utc::now().timestamp_millis(),
            status,
        }
    }

    fn sample_disk(&mut self) -> (u64, u64) {
        self.disks.refresh(true);
        let root = root_mount();
        let pick = |d: &sysinfo::Disk| {
            let total = d.total_space();
            (total.saturating_sub(d.available_space()), total)
        };
        if let Some(d) = self.disks.iter().find(|d| d.mount_point().to_string_lossy() == root) {
            pick(d)
        } else {
            // 找不到主盘时退回第一个非空磁盘
            self.disks
                .iter()
                .find(|d| d.total_space() > 0)
                .map(pick)
                .unwrap_or((0, 0))
        }
    }

    fn sample_network(&mut self) -> (u64, u64) {
        self.networks.refresh(true);
        let mut rx = 0u64;
        let mut tx = 0u64;
        for (name, data) in self.networks.iter() {
            let n = name.to_string_lossy().to_lowercase();
            if n.contains("loopback") || n == "lo" || n.starts_with("lo0") {
                continue;
            }
            rx = rx.saturating_add(data.received());
            tx = tx.saturating_add(data.transmitted());
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.prev_at).as_secs_f64();
        let (out_rx, out_tx) = if elapsed >= 0.2 && self.prev_at.elapsed().as_secs() < 60 {
            (
                diff_rate(rx, self.prev_rx, elapsed),
                diff_rate(tx, self.prev_tx, elapsed),
            )
        } else {
            (0, 0) // 首次或异常长间隔，速率记 0
        };

        self.prev_rx = rx;
        self.prev_tx = tx;
        self.prev_at = now;
        (out_rx, out_tx)
    }

    /// 迟滞：候选状态连续出现 CONFIRM_TIMES 次才真正切换。
    fn resolve_status(&mut self, candidate: PetStatus) -> PetStatus {
        if candidate == self.pending {
            self.pending_count += 1;
        } else {
            self.pending = candidate;
            self.pending_count = 1;
        }
        if self.pending_count >= CONFIRM_TIMES && self.current != self.pending {
            self.current = self.pending;
        }
        self.current
    }
}

fn ratio(used: u64, total: u64) -> f32 {
    if total == 0 {
        0.0
    } else {
        (used as f64 / total as f64 * 100.0) as f32
    }
}

fn diff_rate(curr: u64, prev: u64, elapsed_sec: f64) -> u64 {
    if curr >= prev {
        ((curr - prev) as f64 / elapsed_sec) as u64
    } else {
        0 // 计数器回绕/网卡重置
    }
}

fn net_pressure(bytes_per_sec: u64) -> f32 {
    ((bytes_per_sec as f64 / NET_BASELINE_BPS).min(1.0) * 100.0) as f32
}

fn status_from_pressure(p: f32) -> PetStatus {
    if p >= 85.0 {
        PetStatus::Alert
    } else if p >= 60.0 {
        PetStatus::Busy
    } else {
        PetStatus::Idle
    }
}

fn round2(v: f32) -> f32 {
    (v * 100.0).round() / 100.0
}
