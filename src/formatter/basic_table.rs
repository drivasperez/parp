use super::ReportFormatter;
use crate::report::Report;

use comfy_table::modifiers::{UTF8_ROUND_CORNERS, UTF8_SOLID_INNER_BORDERS};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Table};

pub struct BasicTableSummary;

impl ReportFormatter for BasicTableSummary {
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
                Cell::new(time_taken),
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
