use clap::Parser;
use formatter::basic_table::BasicTableSummary;
use formatter::ReportFormatter;
use psutil::process;
use report::Report;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

mod formatter;
mod report;

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

        let res = BasicTableSummary.to_string(&report)?;

        process_handle.wait()?;

        println!("{res}");
        Ok(())
    }
}
