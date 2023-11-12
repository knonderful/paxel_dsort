use anyhow::bail;
use std::path::PathBuf;

use crate::shell::Shell;
use clap::Parser;

mod dick_sort;
mod progress;
mod shell;

/// Sorts pics from one directory into other ones
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// The path from where the pics will be read
    #[clap(parse(from_os_str))]
    source_dir: PathBuf,

    /// The path to where the pics will be written. This must not be the source_dir!
    #[clap(parse(from_os_str))]
    destination_dir: PathBuf,

    /// If set, the pics will be moved instead copied
    #[clap(short, long, value_parser, default_value_t = false)]
    r#move: bool,

    /// If set, pics in subdirectories will be read, too
    #[clap(short, long, value_parser, default_value_t = false)]
    recursive: bool,

    /// If set, verbose output is created
    #[clap(short, long, value_parser, default_value_t = false)]
    verbose: bool,

    /// If set, neither the directories are created, nor the pics copied or moved
    #[clap(short, long, value_parser, default_value_t = false)]
    dry_run: bool,

    /// If move and clean are active, the empty directories the files were moved from (and all sub directories) are deleted
    #[clap(short, long, value_parser, default_value_t = false)]
    clean: bool,

    /// Log progress of scanning
    #[clap(short, long, value_parser, default_value_t = false)]
    progress: bool,

    /// Format of the path under destination_dir
    #[clap(short, long, value_parser, default_value_t = String::from("[YEAR]/[MONTH]/[DAY]/"))]
    format: String,
}

fn main() -> anyhow::Result<()> {
    let args: Cli = Cli::try_parse()?;

    let mut shell = if args.verbose {
        Shell::Stdout
    } else {
        Shell::Noop
    };

    shell.println(|| format!("Running with {:?}", args));

    if !args.source_dir.is_dir() {
        bail!("source_dir must be a dir");
    }
    if !args.source_dir.exists() {
        bail!("source_dir must exist");
    }

    dick_sort::sort(args, &mut shell)
}
