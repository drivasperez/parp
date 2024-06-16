use std::path::Path;

use crate::report::Report;

pub mod basic_table;

pub trait ReportFormatter {
    fn to_string(&self, report: &Report) -> anyhow::Result<String>;

    fn to_file(&self, report: &Report, path: &Path) -> anyhow::Result<()> {
        let s = self.to_string(report)?;
        std::fs::write(path, s)?;

        Ok(())
    }
}
