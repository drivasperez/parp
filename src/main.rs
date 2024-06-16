use anyhow::ensure;
use clap::Parser;
use comfy_table::modifiers::{UTF8_ROUND_CORNERS, UTF8_SOLID_INNER_BORDERS};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, CellAlignment, Color, Table};
use psutil::process::{self, MemoryInfo};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Parser)]
enum Args {
    #[command(name = "run")]
    Run(Run),

    #[command(name = "graph")]
    Graph(Graph),
}

#[derive(Parser)]
struct Run {
    #[clap(short, long, default_value = "true")]
    memory: bool,
    #[clap(short, long, default_value = "true")]
    cpu: bool,

    #[clap(short, long, default_value = "100")]
    interval_ms: u64,

    #[clap(short, long = "output")]
    output_path: Option<PathBuf>,

    command: Vec<String>,
}

#[derive(Parser)]
struct Graph {
    #[clap(short, long)]
    input_path: PathBuf,

    #[clap(short, long = "braille")]
    use_braille: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Run(run) => {
            println!("Running command: {:?}", run.command);
            println!("Memory: {}", run.memory);
            println!("CPU: {}", run.cpu);
            println!("Interval: {}", run.interval_ms);
            println!("Output path: {:?}", run.output_path);

            let recorder = Runner {
                memory: run.memory,
                cpu: run.cpu,
                interval: Duration::from_millis(run.interval_ms),
                output_path: run.output_path,
                command: run.command,
            };

            println!("Recorder: {:?}", recorder);

            recorder.start_process().unwrap();
        }
        Args::Graph(graph) => {
            println!("Graphing input path: {:?}", graph.input_path);
            println!("Using braille: {}", graph.use_braille);
        }
    }

    Ok(())
}

#[derive(Debug)]
struct Runner {
    memory: bool,
    cpu: bool,
    interval: Duration,
    output_path: Option<PathBuf>,
    command: Vec<String>,
}

impl Runner {
    fn start_process(&self) -> anyhow::Result<()> {
        let Some((command, args)) = self.command.split_first() else {
            anyhow::bail!("Command is empty");
        };

        let time_start = Instant::now();

        let mut process_handle = Command::new(command).args(args).spawn()?;
        let process_id = process_handle.id();

        let mut proc = psutil::process::Process::new(process_id)?;
        println!("Process started with id: {}", process_id);

        let mut entries = Vec::new();

        let time_end = loop {
            use process::Status;

            if matches!(proc.status()?, Status::Dead | Status::Zombie) {
                let end_time = Instant::now();
                break end_time;
            }

            let mem = proc.memory_info()?;
            let cpu = proc.cpu_percent()?;

            entries.push((mem, cpu));

            std::thread::sleep(self.interval);
        };

        let report = Report::new(entries, time_start, time_end);

        let res = TableSummary.to_string(&report)?;

        process_handle.wait()?;

        println!("{res}");
        Ok(())
    }
}

#[derive(Debug)]
struct Report {
    time_start: Instant,
    time_end: Instant,
    memory: Vec<MemoryInfo>,
    cpu: Vec<f32>,
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

trait ReportFormatter {
    fn to_string(&self, report: &Report) -> anyhow::Result<String>;

    fn to_file(&self, report: &Report, path: &Path) -> anyhow::Result<()> {
        let s = self.to_string(report)?;
        std::fs::write(path, &s)?;

        Ok(())
    }
}

pub struct TableSummary;

impl ReportFormatter for TableSummary {
    fn to_string(&self, report: &Report) -> anyhow::Result<String> {
        if report.memory.is_empty() || report.cpu.is_empty() {
            anyhow::bail!("Report was empty");
        }

        let time_taken = humantime::Duration::from(report.time_end - report.time_start).to_string();
        let mut time_table = Table::new();

        time_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_SOLID_INNER_BORDERS)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .add_row(vec![
                Cell::new("Duration").add_attribute(Attribute::Bold),
                Cell::new(&time_taken),
            ]);

        let mut table = Table::new();

        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_SOLID_INNER_BORDERS)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec![
                Cell::new(""),
                Cell::new("CPU (%)").add_attribute(Attribute::Italic),
                Cell::new("Memory (rss)").add_attribute(Attribute::Italic),
                Cell::new("Memory (vms)").add_attribute(Attribute::Italic),
                Cell::new("Memory (shared)").add_attribute(Attribute::Italic),
            ]);

        let peak_cpu = report.max_cpu_percentage();
        let mean_cpu = report.mean_cpu_percentage().unwrap();
        let peak_rss = report.max_rss_memory().unwrap();
        let mean_rss = report.mean_rss_memory().unwrap();
        let peak_vms = report.max_vms_memory().unwrap();
        let mean_vms = report.mean_vms_memory().unwrap();
        let peak_shared = report.max_shared_memory().unwrap();
        let mean_shared = report.mean_shared_memory().unwrap();

        table.add_row(vec![
            Cell::new("Mean".to_string()).add_attribute(Attribute::Bold),
            Cell::new(format!("{:.2}%", mean_cpu)),
            Cell::new(humansize::format_size(mean_rss, humansize::DECIMAL)),
            Cell::new(humansize::format_size(mean_vms, humansize::DECIMAL)),
            Cell::new(humansize::format_size(mean_shared, humansize::DECIMAL)),
        ]);

        table.add_row(vec![
            Cell::new("Peak".to_string()).add_attribute(Attribute::Bold),
            Cell::new(format!("{:.2}%", peak_cpu)),
            Cell::new(humansize::format_size(peak_rss, humansize::DECIMAL)),
            Cell::new(humansize::format_size(peak_vms, humansize::DECIMAL)),
            Cell::new(humansize::format_size(peak_shared, humansize::DECIMAL)),
        ]);

        Ok(format!("{}\n{}", time_table, table))
    }
}
