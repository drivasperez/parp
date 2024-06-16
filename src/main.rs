use clap::Parser;
use formatter::basic_table::BasicTableSummary;
use formatter::textplot::TextplotFormatter;
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
    #[clap(long, default_value = "false")]
    graph_memory: bool,
    #[clap(long, default_value = "false")]
    graph_cpu: bool,

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
            let recorder = Runner {
                graph_memory: run.graph_memory,
                graph_cpu: run.graph_cpu,
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
    graph_memory: bool,
    graph_cpu: bool,
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

        process_handle.wait()?;

        let report = Report::new(entries, time_start, time_end);

        let res = BasicTableSummary.to_string(&report)?;

        println!("{res}");

        if self.graph_memory || self.graph_cpu {
            let graphs =
                TextplotFormatter::new(self.graph_memory, self.graph_cpu).to_string(&report)?;

            println!("{}", graphs);
        }

        Ok(())
    }
}
