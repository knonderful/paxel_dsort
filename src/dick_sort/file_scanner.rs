use std::collections::VecDeque;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use exif::{*, DateTime};
use termion::cursor::DetectCursorPos;
use termion::raw::IntoRawMode;

use crate::Cli;
use crate::dick_sort::{CopyImage, ReadError, SortedDayTime};

pub fn scan(args: &Cli, mut unprocessed_directories: VecDeque<PathBuf>) -> VecDeque<CopyImage> {
    let mut files: VecDeque<CopyImage> = VecDeque::new();

    if args.progress {
// create empty lines for output
        println!();
        println!();
        println!();
        println!();

        let mut progress_screen = std::io::stdout().into_raw_mode().unwrap();
        let result = progress_screen.cursor_pos();
        let line = result.unwrap().1;
        while !unprocessed_directories.is_empty() {
            write!(progress_screen, "{}{}Remaining directories: {}", termion::cursor::Goto(1, line - 3), termion::clear::CurrentLine, unprocessed_directories.len() - 1)
                .expect("could not write to terminal");
            write!(progress_screen, "{}{}      Collected files: {}", termion::cursor::Goto(1, line - 2), termion::clear::CurrentLine, files.len())
                .expect("could not write to terminal");
            find_files(&mut files, &mut unprocessed_directories, args, line).expect("We should be able to read and write files");
            progress_screen.flush().expect("We should be able to flush output ");
        }
        progress_screen.suspend_raw_mode().expect("We should be able to switch back from raw mode");
        println!();
    } else {
        while !unprocessed_directories.is_empty() {
            find_files(&mut files, &mut unprocessed_directories, args, 0).expect("We should be able to read and write files");
            std::thread::yield_now();
        }
    }

    files
}


fn find_files(result: &mut VecDeque<CopyImage>, unprocessed_directories: &mut VecDeque<PathBuf>, args: &Cli, line: u16) -> Result<(), ReadError> {
    // pop next from queue. queue is not empty so it should have an entry
    let dir = unprocessed_directories.pop_front().ok_or(ReadError { msg: "No more entries".to_string() })?;
    if args.progress {
        print!("{}{}       processing dir: {}", termion::cursor::Goto(1, line - 1), termion::clear::CurrentLine, dir.display());
    } else if args.verbose {
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
        } else if args.verbose && !args.progress {
            println!("Error entering path: {:?}: {}", dir, dir_entry_result.err().unwrap());
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
        .flatten()
        .reduce(|l, r| if l > r { r } else { l });

    selected.map_or(Err(ReadError { msg: "No Date Time in file".to_string() }),
                    |sdt| Ok(CopyImage { source: path, date_time: sdt }))
}


fn read_and_validate(exif: &Exif, tag: Tag) -> Option<SortedDayTime> {
    // parse the given tag from the exif
    if let Some(field) = exif.get_field(tag, In::PRIMARY) {
        if let Value::Ascii(ref a) = field.value {
            // parse ascii as DateTime or fail
            if let Ok(new_date) = DateTime::from_ascii(&a[0]) {
                // check if we have a previous value
                return validate_or(Some(SortedDayTime::new(new_date)), None);
            }
        }
    }
    None
}

fn validate_or(new_date: Option<SortedDayTime>, old_date: Option<SortedDayTime>) -> Option<SortedDayTime> {
    if let Some(dt) = &new_date {
        if dt.date_time.year > 0 && dt.date_time.day > 0 && dt.date_time.month > 0 && dt.date_time.day < 32 && dt.date_time.month < 13 {
            return new_date;
        }
    }
    old_date
}
