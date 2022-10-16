#![allow(unused)]

use clap::Parser;
use exif::{DateTime, *};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::process;

/// Sorts pics from one directory into another one
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
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

    /// Optional filter for matching pics. e.g "*.jpg, *.gif, party*"
    #[clap(short, long, value_parser, default_value = "")]
    filter: String,

    /// The sorting pattern. The pics end up in a directory defined by this pattern.
    /// Supported variables are:
    /// {year} the year the pic was taken (e.g. 1971).
    /// {month} the month the pic was taken (e.g. 12).
    /// {day} the day the pic was taken (e.g. 31).
    /// {parent} the directory in which the pic was found (e.g. parties)
    #[clap(short, long, value_parser, default_value = "{year}/{month}/{day}")]
    pattern: String,

    /// If set the date is taken from the files creation dir if no exif:create-date is found
    #[clap(short, long, value_parser, default_value_t = false)]
    use_file_creation_date: bool,

    /// If set verbose output is created
    #[clap(short, long, value_parser, default_value_t = false)]
    verbose: bool,

    /// If set neither the directories are created, nor the pics copied or moved
    #[clap(short, long, value_parser, default_value_t = false)]
    dry_run: bool,
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
    if !args.destination_dir.exists() & &!args.dry_run {
        if args.verbose {
            println!(
                "Creating Destination dir {}",
                args.destination_dir.display()
            );
        }
        let dir_created = fs::create_dir_all(&args.destination_dir);
        match dir_created {
            Ok(n) => {
                if args.verbose {
                    println!("Created {} {:?}", args.destination_dir.display(), n)
                }
            }
            Err(e) => {
                println!(
                    "Could not create destination dir {} {}",
                    args.destination_dir.display(),
                    e.to_string()
                );
                process::exit(1);
            }
        }
    }

    recurse_dir(&args.source_dir, args.verbose);
    // here we have source and destination
}

fn recurse_dir(path: &PathBuf, verbose: bool) {
    if verbose {
        println!("Processing dir {:?}", path);
    }

    let paths = fs::read_dir(path);
    match paths {
        Ok(p) => {
            for path in p {
                match path {
                    Ok(f) => {
                        if f.path().is_dir() {
                            recurse_dir(&f.path(), verbose);
                        } else if verbose {
                            println!("Found: {}", f.path().display())
                        }
                        print_exif(&f.path(), verbose)
                    }
                    Err(e) => {
                        println!("Error: {}", e.to_string());
                    }
                }
            }
        }
        Err(e) => {
            println!("Error entering path: {:?}: {}", path, e.to_string());
        }
    }
}

fn print_exif(path: &PathBuf, verbose: bool) {
    let file = std::fs::File::open(path).unwrap();
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader);
    match exif {
        Ok(x) => match x.get_field(Tag::DateTime, In::PRIMARY) {
            Some(date_time) => match date_time.value {
                Value::Ascii(ref a) => {
                    let dt = DateTime::from_ascii(&a[0]);
                    if verbose {
                        println!("Date Time: {:?}", dt);
                    }
                }
                _ => eprintln!("hu?"),
            },
            None => eprintln!("Orientation tag is missing"),
        },
        Err(e) => {
            println!("Error: {}", e.to_string());
        }
    }
}
