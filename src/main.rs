#![allow(unused)]

use std::collections::VecDeque;
use clap::{arg, Parser};
use exif::{DateTime, *};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::process;
use termion;
use termion::cursor::DetectCursorPos;
use termion::raw::IntoRawMode;

use ctrlc;

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

    /// If set verbose output is created
    #[clap(short, long, value_parser, default_value_t = false)]
    verbose: bool,

    /// If set neither the directories are created, nor the pics copied or moved
    #[clap(short, long, value_parser, default_value_t = false)]
    dry_run: bool,

    /// Log progress of scanning
    #[clap(short, long, value_parser, default_value_t = false)]
    progress: bool,

}

fn main() {

    // doesn't work :(
    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
    }).expect("Error setting Ctrl-C handler");

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

    if args.progress {
        // create empty lines for output
        println!();
        println!();
        println!();
        println!();

        let mut stdout = std::io::stdout().into_raw_mode().unwrap();
        let result = stdout.cursor_pos();
        let line = result.unwrap().1;
        while !unprocessed_directories.is_empty() {
            print!("{}{}Remaining directories: {}", termion::cursor::Goto(1, line - 3), termion::clear::CurrentLine, unprocessed_directories.len() - 1);
            print!("{}{}      Collected files: {}", termion::cursor::Goto(1, line - 2), termion::clear::CurrentLine, files.len());
            find_files(&mut files, &mut unprocessed_directories, &args, line);
            std::thread::yield_now();
        }
        let result1 = stdout.suspend_raw_mode();
    } else {
        while !unprocessed_directories.is_empty() {
            find_files(&mut files, &mut unprocessed_directories, &args, 0);
            std::thread::yield_now();
        }
    }
    println!();
    while !files.is_empty() {
        if args.r#move {
            match move_file(&args, &mut files) {
                Ok(true) => {
                    // count moves
                }
                Err(e) => {
                    eprintln!("Failed {:?}", e);
                    copy_and_count(&args, &mut files);
                }
                def => {
                    // count same file
                }
            }
        } else {
            copy_and_count(&args, &mut files);
        }
    }
}

fn copy_and_count(args: &Cli, mut files: &mut VecDeque<CopyImage>) {
    match copy_file(&args, &mut files) {
        Ok(true) => {
            // count copies
        }
        Err(e) => {
            eprintln!("Failed copy {:?}", e);
        }
        def => {
            // count same file
        }
    }
}

fn copy_file(args: &Cli, files: &mut VecDeque<CopyImage>) -> Result<bool, ReadError> {
    let (mut path, image) = build_and_create_path(args, files)?;


    if !image.source.eq(&path) {
        return if args.dry_run {
            println!("Would copy from {:?} to {:?}", image.source, path);
            Ok(false)
        } else {
            let size = fs::copy(image.source, &path).map_err(|err| ReadError { msg: err.to_string() })?;
            if args.verbose {
                println!("Copied {:?} bytes to {:?}", size, path);
            }
            Ok(true)
        };
    }
    Ok(false)
}

fn move_file(args: &Cli, files: &mut VecDeque<CopyImage>) -> Result<bool, ReadError> {
    let (mut path, image) = build_and_create_path(args, files)?;

    if !image.source.eq(&path) {
        return if args.dry_run {
            println!("Would move from {:?} to {:?}", image.source, path);
            Ok(false)
        } else {
            let result = fs::rename(image.source, &path).map_err(|err| ReadError { msg: err.to_string() })?;
            let size = fs::metadata(&path).unwrap().len();
            if args.verbose {
                println!("Moved {:?} bytes to {:?}", size, path);
            }
            Ok(true)
        };
    }
    Ok(false)
}

fn build_and_create_path(args: &Cli, files: &mut VecDeque<CopyImage>) -> Result<(PathBuf, CopyImage), ReadError> {
    let destination = args.destination_dir.to_str().ok_or(ReadError { msg: "destination dir has no string".to_string() })?;
    let image = files.pop_front().ok_or(ReadError { msg: "No more elements in queue".to_string() })?;
    let name = image.source.file_name().ok_or(ReadError { msg: "File has no filename".to_string() })?;

    let mut path = PathBuf::new();
    path.push(destination);
    path.push(format!("{:04}", image.date_time.year));
    path.push(format!("{:02}", image.date_time.month));
    path.push(format!("{:02}", image.date_time.day));
    if !(args.dry_run) {
        let result = fs::create_dir_all(&path).map_err(|err| ReadError { msg: err.to_string() })?;
    }
    path.push(name);
    Ok((path, image))
}


fn find_files(result: &mut VecDeque<CopyImage>, unprocessed_directories: &mut VecDeque<PathBuf>, args: &Cli, line: u16) -> Result<(), ReadError> {
    // pop next from queue. queue is not empty so it should have an entry
    let dir = unprocessed_directories.pop_front().ok_or(ReadError { msg: "No more entries".to_string() })?;
    if args.verbose {
        println!("Processing dir {:?}", dir);
    }
    if args.progress {
        print!("{}{}       processing dir: {}", termion::cursor::Goto(1, line - 1), termion::clear::CurrentLine, dir.display());
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
            if let Some(ext) = entry.path().extension() {
                if ext.to_ascii_lowercase().eq("jpg")
                    || ext.to_ascii_lowercase().eq("jpeg")
                    || ext.to_ascii_lowercase().eq("heic") {
                    // we have a jpegish file, so try to read exif.

                    if args.progress {
                        print!("{}{}      processing file: {}", termion::cursor::Goto(1, line), termion::clear::CurrentLine, entry.path().display());
                    }
                    if let Ok(image) = read_exif(entry.path(), &args) {
                        result.push_back(image);
                    }
                }
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
    let file = File::open(&path).map_err(|err| ReadError { msg: err.to_string() })?;
    let mut buf_reader = std::io::BufReader::new(&file);
    let exif_reader = Reader::new();
    // read exif or fail
    let exif = exif_reader.read_from_container(&mut buf_reader).map_err(|err| ReadError { msg: err.to_string() })?;
    // get date time field or fail
    let tag = Tag::DateTimeOriginal;
    let previous = read_and_choose_oldest(&exif, Tag::DateTimeOriginal, None);
    let previous = read_and_choose_oldest(&exif, Tag::DateTimeDigitized, previous);
    let previous = read_and_choose_oldest(&exif, Tag::DateTime, previous);
    let selected = read_and_choose_oldest(&exif, Tag::GPSDateStamp, previous)
        .ok_or(ReadError { msg: "No Date Time in file".to_string() })?;

    Ok(CopyImage { source: path, date_time: selected })
}

fn read_and_choose_oldest(exif: &Exif, tag: Tag, previous: Option<DateTime>) -> Option<DateTime> {
    // parse the given tag from the exif
    if let Some(field) = exif.get_field(tag, In::PRIMARY) {
        if let Value::Ascii(ref a) = field.value {
            // parse ascii as DateTime or fail
            if let Ok(new_date) = DateTime::from_ascii(&a[0]) {
                // check if we have a previous value
                if !(previous.is_none()) {
                    // decide which one is the older one
                    let old_date = previous.unwrap();
                    if new_date.year > old_date.year {
                        return Some(old_date);
                    } else if new_date.year < old_date.year {
                        return validate_or(Some(new_date), Some(old_date));
                    }
                    // same year
                    if new_date.month > old_date.month {
                        return Some(old_date);
                    } else if new_date.month < old_date.month {
                        return validate_or(Some(new_date), Some(old_date));
                    }
                    // same month
                    if new_date.day > old_date.day {
                        return Some(old_date);
                    } else if new_date.day < old_date.day {
                        return validate_or(Some(new_date), Some(old_date));
                    }
                }
                return Some(new_date);
            }
        }
    }
    return previous;
}

fn validate_or(new_date: Option<DateTime>, old_date: Option<DateTime>) -> Option<DateTime> {
    if let Some(dt) = &new_date {
        if dt.year > 0 && dt.day > 0 && dt.month > 0 && dt.day < 32 && dt.month < 13 {
            return new_date;
        }
    }
    return old_date;
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
