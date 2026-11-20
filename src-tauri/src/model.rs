use serde::Serialize;

/// 前后端共享的系统状态快照。字段名用 camelCase 与 TS 对齐。
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub mem_used: u64,
    pub mem_total: u64,
    pub mem_usage: f32,
    pub disk_used: u64,
    pub disk_total: u64,
    pub disk_usage: f32,
    pub net_rx: u64,
    pub net_tx: u64,
    pub sampled_at: i64,
    pub status: PetStatus,
}

#[derive(Clone, Copy, Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PetStatus {
    Idle,
    Busy,
    Alert,
}
