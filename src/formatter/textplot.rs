use super::ReportFormatter;
use rgb::RGB8;
use textplots::{Chart, ColorPlot, Shape};

pub struct TextplotFormatter {
    memory: bool,
    cpu: bool,
}

impl TextplotFormatter {
    pub fn new(memory: bool, cpu: bool) -> Self {
        Self { memory, cpu }
    }
}

impl ReportFormatter for TextplotFormatter {
    fn to_string(&self, report: &crate::report::Report) -> anyhow::Result<String> {
        let mut output = Vec::new();
        if self.memory {
            let memory_data = report
                .memory
                .iter()
                .map(|x| x.rss() as f32)
                .enumerate()
                .map(|(i, x)| (i as f32, x / 1024.0 / 1024.0))
                .collect::<Vec<(f32, f32)>>();

            let shape = Shape::Lines(&memory_data);
            let mut chart = Chart::new(180, 90, 0.0, report.memory.len() as f32);
            let chart = chart.linecolorplot(&shape, RGB8::new(255, 0, 0));

            chart.axis();
            chart.figures();

            let chart_str = chart.to_string();

            output.push(chart_str);
        }

        if self.cpu {
            let cpu_data = report
                .cpu
                .iter()
                .map(|x| *x)
                .enumerate()
                .map(|(i, x)| (i as f32, x))
                .collect::<Vec<(f32, f32)>>();

            let shape = Shape::Lines(&cpu_data);
            let mut chart = Chart::new(180, 90, 0.0, report.cpu.len() as f32);
            let chart = chart.linecolorplot(&shape, RGB8::new(0, 255, 0));

            chart.axis();
            chart.figures();

            let chart_str = chart.to_string();

            output.push(chart_str);
        }

        Ok(output.join("\n\n"))
    }
}
