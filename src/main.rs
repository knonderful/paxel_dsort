
mod dicksort;
use clap::{ Parser};
use std::process;
use std::fs;
use std::path::PathBuf;
use dicksort::ReadError;



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

    /// If set the pics will be moved instead copied
    #[clap(short, long, value_parser, default_value_t = false)]
    r#move: bool,

    /// If set pics in subdirectories will be read, too
    #[clap(short, long, value_parser, default_value_t = false)]
    recursive: bool,

    /// If set verbose output is created
    #[clap(short, long, value_parser, default_value_t = false)]
    verbose: bool,

    /// If set neither the directories are created, nor the pics copied or moved
    #[clap(short, long, value_parser, default_value_t = false)]
    dry_run: bool,

    /// If move and clean are active, the empty directories the files were moved from are deleted
    #[clap(short, long, value_parser, default_value_t = false)]
    clean: bool,

    /// Log progress of scanning
    #[clap(short, long, value_parser, default_value_t = false)]
    progress: bool,

}

fn main() {

    let args: Cli = Cli::parse();
    if args.verbose {
        println!("Running with {:?}", args);
    }
    if !args.source_dir.is_dir() {
        println!("source_dir must be a dir");
        process::exit(1);
    }
    if !args.source_dir.exists() {
        println!("source_dir must exist");
        process::exit(1);
    }
    let created_dir = create_target_dir(&args);
    if created_dir.is_err() {
        eprintln!("Could not create destination dir {} {}", &args.destination_dir.display(), created_dir.err().unwrap().msg);
        process::exit(1);
    }
    dicksort::sort(args);
}

fn create_target_dir(args: &Cli) -> Result<(), ReadError> {
    if !args.destination_dir.exists() & &!args.dry_run {
        if args.verbose {
            println!("Creating Destination dir {}", args.destination_dir.display());
        }
        let dir_created = fs::create_dir_all(&args.destination_dir);
        return match dir_created {
            Ok(n) => {
                if args.verbose {
                    println!("Created {} {:?}", args.destination_dir.display(), n)
                }
                Ok(())
            }
            Err(e) => {
                Err(ReadError { msg: e.to_string() })
            }
        };
    }
    Ok(())
}
