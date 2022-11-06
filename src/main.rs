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

#[derive(Debug)]
pub struct ReadError {
    pub msg: String,
}

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
    let created_dir = create_target_dir(&args);
    if created_dir.is_err() {
        eprintln!("Could not create destination dir {} {}", &args.destination_dir.display(), created_dir.err().unwrap().msg);
        process::exit(1);
    }

    let mut unprocessed_directories: VecDeque<PathBuf> = VecDeque::new();
    let mut files: VecDeque<CopyImage> = VecDeque::new();

    // add source dir in the queue
    unprocessed_directories.push_back(args.source_dir.clone());
    println!("Collecting files");
    while !unprocessed_directories.is_empty() {
        find_files(&mut files, &mut unprocessed_directories, &args);
    }

    println!("Found {:?} files with valid date time in exif", files.len());

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


fn find_files(result: &mut VecDeque<CopyImage>, unprocessed_directories: &mut VecDeque<PathBuf>, args: &Cli) -> Result<(), ReadError> {
    // pop next from queue. queue is not empty so it should have an entry
    let dir = unprocessed_directories.pop_front().ok_or(ReadError { msg: "No more entries".to_string() })?;
    if args.verbose {
        println!("Processing dir {:?}", dir);
    }

    // read the files of the dir
    let read_dir_result = fs::read_dir(&dir).map_err(|err| ReadError { msg: err.to_string() })?;

    for dir_entry_result in read_dir_result {
        if let Ok(entry) = dir_entry_result {
            if entry.path().is_dir() {
                // we have a dir, we ignore it if not recursive is active
                if args.recursive {
                    unprocessed_directories.push_back(entry.path());
                }
                continue;
            }
            // we have a file, so try to read exif.
            if let Ok(image) = read_exif(entry.path(), &args) {
                result.push_back(image);
            }
        } else {
            if args.verbose {
                println!("Error entering path: {:?}: {}", dir, dir_entry_result.err().unwrap().to_string());
            }
        }
    }
    Ok(())
}

fn read_exif(path: PathBuf, p1: &Cli) -> Result<CopyImage, ReadError> {
    // open file or fail
    let file = std::fs::File::open(&path).map_err(|err| ReadError { msg: err.to_string() })?;
    let mut buf_reader = std::io::BufReader::new(&file);
    let exif_reader = exif::Reader::new();
    // read exif or fail
    let exif = exif_reader.read_from_container(&mut buf_reader).map_err(|err| ReadError { msg: err.to_string() })?;
    // get date time field or fail
    let date_time = exif.get_field(Tag::DateTime, In::PRIMARY).ok_or(ReadError { msg: "Missing DateTime in Primary".to_string() })?;

    // check if the value is ascii
    if let Value::Ascii(ref a) = date_time.value {
        // parse ascii as DateTime or fail
        let dt = DateTime::from_ascii(&a[0]).map_err(|err| ReadError { msg: err.to_string() })?;
        // check if the values are in the range (roughly)
        if dt.year > 0 && dt.day > 0 && dt.month > 0 && dt.day < 32 && dt.month < 13 {
            Ok(CopyImage { source: path, date_time: dt })
        } else {
            Err(ReadError { msg: "Date Time Values are invalid".to_string() })
        }
    } else {
        Err(ReadError { msg: "Date Time Field is not ascii".to_string() })
    }
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

fn print_tags(exif: &Exif) {
    for f in exif.fields() {
        eprintln!("{} {} {}", f.tag, f.ifd_num, f.display_value().with_unit(exif));
    }
}
