use anyhow::Context;
use std::io::{Stdout, Write};
use std::path::Path;
use termion::raw::RawTerminal;

pub trait ProgressReport {
    fn set_remaining_dirs(&mut self, remaining_dirs: usize) -> anyhow::Result<()>;
    fn set_collected_files(&mut self, collected_files: usize) -> anyhow::Result<()>;
    fn set_current_dir(&mut self, dir: &Path) -> anyhow::Result<()>;
    fn set_current_file(&mut self, file: &Path) -> anyhow::Result<()>;
    fn flush(&mut self) -> anyhow::Result<()>;
}

pub struct NoopProgressReport;

impl ProgressReport for NoopProgressReport {
    fn set_remaining_dirs(&mut self, _remaining_dirs: usize) -> anyhow::Result<()> {
        Ok(())
    }

    fn set_collected_files(&mut self, _collected_files: usize) -> anyhow::Result<()> {
        Ok(())
    }

    fn set_current_dir(&mut self, _dir: &Path) -> anyhow::Result<()> {
        Ok(())
    }

    fn set_current_file(&mut self, _file: &Path) -> anyhow::Result<()> {
        Ok(())
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct TerminalProgressReport {
    term: RawTerminal<Stdout>,
    base_line: u16,
}

impl TerminalProgressReport {
    pub fn new() -> anyhow::Result<Self> {
        use termion::raw::IntoRawMode as _;
        let mut term = std::io::stdout().into_raw_mode()?;

        // Create empty lines for output
        for _ in 0..4 {
            writeln!(term)?;
        }

        use termion::cursor::DetectCursorPos as _;
        let (_, base_line) = term.cursor_pos()?;

        Ok(Self { term, base_line })
    }
}

impl ProgressReport for TerminalProgressReport {
    fn set_remaining_dirs(&mut self, remaining_dirs: usize) -> anyhow::Result<()> {
        write!(
            self.term,
            "{}{}Remaining directories: {}",
            termion::cursor::Goto(1, self.base_line - 3),
            termion::clear::CurrentLine,
            remaining_dirs
        )?;
        Ok(())
    }

    fn set_collected_files(&mut self, collected_files: usize) -> anyhow::Result<()> {
        write!(
            self.term,
            "{}{}      Collected files: {}",
            termion::cursor::Goto(1, self.base_line - 2),
            termion::clear::CurrentLine,
            collected_files
        )?;
        Ok(())
    }

    fn set_current_dir(&mut self, dir: &Path) -> anyhow::Result<()> {
        write!(
            self.term,
            "{}{}       processing dir: {:?}",
            termion::cursor::Goto(1, self.base_line - 1),
            termion::clear::CurrentLine,
            dir
        )?;
        Ok(())
    }

    fn set_current_file(&mut self, file: &Path) -> anyhow::Result<()> {
        write!(
            self.term,
            "{}{}      processing file: {:?}",
            termion::cursor::Goto(1, self.base_line),
            termion::clear::CurrentLine,
            file
        )?;
        Ok(())
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        self.term
            .flush()
            .context("We should be able to flush output ")?;
        Ok(())
    }
}
