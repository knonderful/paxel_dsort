use std::collections::VecDeque;
use exif::{DateTime, *};
use std::fs;
use std::fs::File;
use std::io::{Write};
use std::path::PathBuf;
use termion;
use termion::cursor::DetectCursorPos;
use termion::raw::{IntoRawMode};
use crate::Cli;


#[derive(Debug)]
pub struct CopyImage {
    pub source: PathBuf,
    pub date_time: DateTime,
}


#[derive(Debug)]
pub struct ReadError {
    pub msg: String,
}

pub fn sort(args: Cli) {
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

        let mut termion_screen = std::io::stdout().into_raw_mode().unwrap();
        let result = termion_screen.cursor_pos();
        let line = result.unwrap().1;
        while !unprocessed_directories.is_empty() {
            write!(termion_screen, "{}{}Remaining directories: {}", termion::cursor::Goto(1, line - 3), termion::clear::CurrentLine, unprocessed_directories.len() - 1)
                .expect("could not write to terminal");
            write!(termion_screen, "{}{}      Collected files: {}", termion::cursor::Goto(1, line - 2), termion::clear::CurrentLine, files.len())
                .expect("could not write to terminal");
            find_files(&mut files, &mut unprocessed_directories, &args, line).expect("We should be able to read and write files");
            termion_screen.flush().expect("We should be able to flush output ");
        }
        termion_screen.suspend_raw_mode().expect("We should be able to switch back from raw mode");
    } else {
        while !unprocessed_directories.is_empty() {
            find_files(&mut files, &mut unprocessed_directories, &args, 0).expect("We should be able to read and write files");
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
                _def => {
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
        _def => {
            // count same file
        }
    }
}

fn copy_file(args: &Cli, files: &mut VecDeque<CopyImage>) -> Result<bool, ReadError> {
    let (path, image) = build_and_create_path(args, files)?;


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
    let (path, image) = build_and_create_path(args, files)?;

    if !image.source.eq(&path) {
        return if args.dry_run {
            println!("Would move from {:?} to {:?}", image.source, path);
            Ok(false)
        } else {
            fs::rename(image.source, &path).map_err(|err| ReadError { msg: err.to_string() })?;
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
        fs::create_dir_all(&path).map_err(|err| ReadError { msg: err.to_string() })?;
    }
    path.push(name);
    Ok((path, image))
}


fn find_files(result: &mut VecDeque<CopyImage>, unprocessed_directories: &mut VecDeque<PathBuf>, args: &Cli, line: u16) -> Result<(), ReadError> {
    // pop next from queue. queue is not empty so it should have an entry
    let dir = unprocessed_directories.pop_front().ok_or(ReadError { msg: "No more entries".to_string() })?;
    if args.verbose {
        println!("Processing dir {:?}", dir);
    } else if args.progress {
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
                    if let Ok(image) = read_exif(entry.path()) {
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

fn read_exif(path: PathBuf) -> Result<CopyImage, ReadError> {
    // open file or fail
    let file = File::open(&path).map_err(|err| ReadError { msg: err.to_string() })?;
    let mut buf_reader = std::io::BufReader::new(&file);
    let exif_reader = Reader::new();
    // read exif or fail
    let exif = exif_reader.read_from_container(&mut buf_reader).map_err(|err| ReadError { msg: err.to_string() })?;
    // get date time field or fail
    let orig = read_and_validate(&exif, Tag::DateTimeOriginal);
    let digi = read_and_validate(&exif, Tag::DateTimeDigitized);
    let create = read_and_validate(&exif, Tag::DateTime);
    let gps = read_and_validate(&exif, Tag::GPSDateStamp);

    let selected = [orig, digi, create, gps]
        .into_iter()
        .filter_map(|x| x)
        .reduce(|l, r| if younger_than(&l, &r) { r } else { l });

    selected.map_or(Err(ReadError { msg: "No Date Time in file".to_string() }),
                    |dt| Ok(CopyImage { source: path, date_time: dt }))
}

fn younger_than(a: &DateTime, b: &DateTime) -> bool {
    if a.year > b.year {
        return true;
    } else if a.year < b.year {
        return false;
    }
    // same year
    if a.month > b.month {
        return true;
    } else if a.month < b.month {
        return false;
    }
    // same month
    if a.day > b.day {
        return true;
    } else if a.day < b.day {
        return false;
    }
    //equal
    false
}

fn read_and_validate(exif: &Exif, tag: Tag) -> Option<DateTime> {
    // parse the given tag from the exif
    if let Some(field) = exif.get_field(tag, In::PRIMARY) {
        if let Value::Ascii(ref a) = field.value {
            // parse ascii as DateTime or fail
            if let Ok(new_date) = DateTime::from_ascii(&a[0]) {
                // check if we have a previous value
                return validate_or(Some(new_date), None);
            }
        }
    }
    None
}

fn validate_or(new_date: Option<DateTime>, old_date: Option<DateTime>) -> Option<DateTime> {
    if let Some(dt) = &new_date {
        if dt.year > 0 && dt.day > 0 && dt.month > 0 && dt.day < 32 && dt.month < 13 {
            return new_date;
        }
    }
    return old_date;
}


