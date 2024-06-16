use psutil::process::MemoryInfo;
use std::time::Instant;

#[derive(Debug)]
pub struct Report {
    pub time_start: Instant,
    pub time_end: Instant,
    pub memory: Vec<MemoryInfo>,
    pub cpu: Vec<f32>,
}

impl Report {
    pub fn new(records: Vec<(MemoryInfo, f32)>, time_start: Instant, time_end: Instant) -> Self {
        let (memory, cpu) = records.into_iter().unzip();

        Self {
            time_start,
            time_end,
            memory,
            cpu,
        }
    }

    pub fn max_cpu_percentage(&self) -> f32 {
        self.cpu.iter().fold(0.0f32, |a, &b| a.max(b))
    }

    pub fn mean_cpu_percentage(&self) -> Option<f32> {
        if self.cpu.is_empty() {
            return None;
        }
        let total: f32 = self.cpu.iter().sum();
        let mean = total / self.cpu.len() as f32;

        Some(mean)
    }

    pub fn max_shared_memory(&self) -> Option<u64> {
        self.memory.iter().map(|x| x.shared()).max()
    }

    pub fn mean_shared_memory(&self) -> Option<u64> {
        if self.memory.is_empty() {
            return None;
        }

        let total: u64 = self.memory.iter().map(|x| x.shared()).sum();
        let mean = total / self.memory.len() as u64;

        Some(mean)
    }

    pub fn max_rss_memory(&self) -> Option<u64> {
        self.memory.iter().map(|x| x.rss()).max()
    }

    pub fn mean_rss_memory(&self) -> Option<u64> {
        if self.memory.is_empty() {
            return None;
        }

        let total: u64 = self.memory.iter().map(|x| x.rss()).sum();
        let mean = total / self.memory.len() as u64;

        Some(mean)
    }

    pub fn max_vms_memory(&self) -> Option<u64> {
        self.memory.iter().map(|x| x.vms()).max()
    }

    pub fn mean_vms_memory(&self) -> Option<u64> {
        if self.memory.is_empty() {
            return None;
        }

        let total: u64 = self.memory.iter().map(|x| x.vms()).sum();
        let mean = total / self.memory.len() as u64;

        Some(mean)
    }
}
