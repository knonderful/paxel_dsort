#![allow(unused)]

use std::collections::VecDeque;
use clap::Parser;
use exif::{DateTime, *};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::process;

#[derive(Debug)]
pub struct CopyImage {
    pub source: PathBuf,
    pub date_time: DateTime,
}

impl CopyImage {}

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

    /// If set verbose output is created
    #[clap(long, value_parser, default_value_t = false)]
    print_exif_dates: bool,

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

    let mut unprocessed_directories: VecDeque<PathBuf> = VecDeque::new();
    let mut files: VecDeque<CopyImage> = VecDeque::new();
    let mut missing_exif: i32 = 0;
    let mut has_exif: i32 = 0;
    let mut total_dirs: i32 = 0;
    let mut total_files: i32 = 0;

    // add surce dir in the queue
    unprocessed_directories.push_back(args.source_dir);
    println!("Collecting files");
    while !unprocessed_directories.is_empty() {
        total_dirs += 1;
        let dir = unprocessed_directories.pop_front().unwrap();
        {
            if args.verbose {
                println!("Processing dir {:?}", dir);
            }
            let read_dir_result = fs::read_dir(&dir);
            match read_dir_result {
                Ok(paths) => {
                    for dir_entry_result in paths {
                        match dir_entry_result {
                            Ok(dir_entry) => {
                                if dir_entry.path().is_dir() {
                                    if args.recursive {
                                        unprocessed_directories.push_back(dir_entry.path());
                                    }
                                    continue;
                                }
                                total_files += 1;
                                let open_result = std::fs::File::open(dir_entry.path());
                                match open_result {
                                    Ok(file) => {
                                        let mut buf_reader = std::io::BufReader::new(&file);
                                        let exif_reader = exif::Reader::new();
                                        let exif_result = exif_reader.read_from_container(&mut buf_reader);
                                        match exif_result {
                                            Ok(exif) => match exif.get_field(Tag::DateTime, In::PRIMARY) {
                                                Some(date_time) => {
                                                    match date_time.value {
                                                        Value::Ascii(ref a) => {
                                                            let dt = DateTime::from_ascii(&a[0]);
                                                            if args.print_exif_dates {
                                                                println!("{}: {:?}", dir_entry.path().display(), dt);
                                                            }
                                                            match dt {
                                                                Ok(dt) => {
                                                                    if dt.year > 0 && dt.day > 0 && dt.month > 0 && dt.day < 32 && dt.month < 13 {
                                                                        files.push_back(CopyImage { source: dir_entry.path(), date_time: dt });
                                                                        has_exif += 1;
                                                                    } else {
                                                                        if args.verbose {
                                                                            eprintln!("invalid date");
                                                                            print_tags(&exif);
                                                                        }
                                                                    }
                                                                }
                                                                _ => {
                                                                    if args.verbose {
                                                                        eprintln!("Could not parse date_time from exif");
                                                                        print_tags(&exif);
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        _ => {
                                                            if args.verbose {
                                                                eprintln!(" ERR date time value {:?}", file);
                                                                print_tags(&exif);
                                                            }
                                                        }
                                                    }
                                                }
                                                None => {
                                                    if args.verbose {
                                                        eprintln!(" ERR date time missing {:?}", file);
                                                        print_tags(&exif);
                                                    }
                                                    missing_exif += 1;
                                                }
                                            }
                                            Err(e) => {
                                                if args.verbose {
                                                    eprintln!(" ERR exif missing {:?}", file);
                                                }
                                                missing_exif += 1;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        if args.verbose {
                                            eprintln!(" ERR can't open dir {:?}", dir_entry.path());
                                        }
                                        missing_exif += 1;
                                    }
                                }
                            }
                            Err(e) => {
                                if args.verbose {
                                    println!("Error: {}", e.to_string());
                                }
                                missing_exif += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    if args.verbose {
                        println!("Error entering path: {:?}: {}", dir, e.to_string());
                    }
                    missing_exif += 1;
                }
            }
        }
    }

    println!("Found {} files in {} dirs with {} exif dates", total_files, total_dirs, has_exif);
    println!("There were {} files without exif ", missing_exif);
    if args.verbose {
        println!("Result: {:?}", files);
    }

    let destination = args.destination_dir.to_str().unwrap();
    let mut total = 0u64;
    let mut copy = 0u32;
    while !files.is_empty() {
        let target = files.pop_front();
        match target {
            Some(image) => {
                match image.source.file_name() {
                    Some(name) => {
                        let mut path = PathBuf::new();
                        path.push(destination);
                        path.push(image.date_time.year.to_string());
                        path.push(image.date_time.month.to_string());
                        path.push(image.date_time.day.to_string());
                        if !args.dry_run {
                            let result = fs::create_dir_all(&path);
                            match result {
                                Ok(value) => {}
                                Err(e) => {
                                    eprintln!("Could not create {:?}", path);
                                    continue;
                                }
                            }
                        } else {
                            println!("Would create {:?}", path);
                        }

                        path.push(name);

                        if !args.dry_run {
                            if !args.r#move {
                                let result = fs::copy(image.source, &path);
                                match result {
                                    Ok(value) => {
                                        if args.verbose {
                                            println!("Copied {:?} bytes to {:?}", value, path);
                                        }
                                        total += value;
                                        copy += 1;
                                    }
                                    Err(e) => {}
                                }
                            } else {
                                let result = fs::rename(image.source, &path);
                                match result {
                                    Ok(value) => {
                                        let size = fs::metadata(&path).unwrap().len();
                                        total += size;
                                        if args.verbose {
                                            println!("Moved {:?} bytes to {:?}", size, path);
                                        }
                                        copy += 1;
                                    }
                                    Err(e) => {}
                                }
                            }
                        } else {
                            println!("Would copy from {:?} to {:?}", image.source, path);
                        }
                    }
                    None => { eprintln!("File without name") }
                }
            }
            _ => eprintln!("woot?")
        }
    }

    println!("Copied or moved {} files with {} bytes", copy, total);
}

fn print_tags(exif: &Exif) {
    for f in exif.fields() {
        eprintln!("{} {} {}", f.tag, f.ifd_num, f.display_value().with_unit(exif));
    }
}
