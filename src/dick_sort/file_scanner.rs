use anyhow::{anyhow, Context};
use std::collections::VecDeque;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

use exif::{DateTime, *};

use crate::dick_sort::{CopyImage, ReadError, SortedDayTime};
use crate::progress::{NoopProgressReport, ProgressReport, TerminalProgressReport};
use crate::shell::Shell;

pub fn scan(
    source_dir: PathBuf,
    shell: &mut Shell,
    show_progress: bool,
    recursive: bool,
) -> anyhow::Result<VecDeque<CopyImage>> {
    let mut unprocessed_directories: VecDeque<PathBuf> = VecDeque::new();
    unprocessed_directories.push_back(source_dir);

    let mut files: VecDeque<CopyImage> = VecDeque::new();

    let mut progress: Box<dyn ProgressReport> = if show_progress {
        Box::new(TerminalProgressReport::new().context("Failed to create progress report.")?)
    } else {
        Box::new(NoopProgressReport)
    };

    while !unprocessed_directories.is_empty() {
        progress.set_remaining_dirs(unprocessed_directories.len() - 1)?;
        progress.set_collected_files(files.len())?;
        find_files(
            &mut files,
            &mut unprocessed_directories,
            shell,
            progress.as_mut(),
            recursive,
        )?;
        std::thread::yield_now();
    }

    Ok(files)
}

fn find_files(
    result: &mut VecDeque<CopyImage>,
    unprocessed_directories: &mut VecDeque<PathBuf>,
    shell: &mut Shell,
    progress: &mut dyn ProgressReport,
    recursive: bool,
) -> anyhow::Result<()> {
    // pop next from queue. queue is not empty so it should have an entry
    let dir = unprocessed_directories
        .pop_front()
        .ok_or(anyhow!("No more entries"))?;

    progress.set_current_dir(&dir)?;
    shell.println(|| format!("Processing dir {:?}", dir));

    // read the files of the dir
    let read_dir_result =
        fs::read_dir(&dir).with_context(|| format!("Failed to read directory: {:?}", &dir))?;

    for dir_entry_result in read_dir_result {
        let entry = match dir_entry_result {
            Ok(entry) => entry,
            Err(err) => {
                shell.println(|| format!("Error entering path: {:?}: {}", dir, err));
                continue;
            }
        };

        if entry.path().is_dir() {
            // we have a dir, we ignore it if not recursive is active
            if recursive {
                unprocessed_directories.push_back(entry.path());
            }
            continue;
        }

        let path = entry.path();
        let Some(ext) = path.extension() else {
            continue;
        };

        let ext_lower_case = ext.to_ascii_lowercase();
        if ["jpg", "jpeg", "heic"]
            .iter()
            .any(|val| ext_lower_case.eq(val))
        {
            // we have a jpegish file, so try to read exif.

            progress.set_current_file(&path)?;
            if let Ok(image) = read_exif(path) {
                // TODO: Handle error case
                result.push_back(image);
            }
        }
    }
    Ok(())
}

fn read_exif(path: PathBuf) -> Result<CopyImage, ReadError> {
    // open file or fail
    let file = File::open(&path).map_err(|err| ReadError {
        msg: err.to_string(),
    })?;
    let mut buf_reader = std::io::BufReader::new(&file);
    let exif_reader = Reader::new();
    // read exif or fail
    let exif = exif_reader
        .read_from_container(&mut buf_reader)
        .map_err(|err| ReadError {
            msg: err.to_string(),
        })?;
    // get date time field or fail
    let orig = read_and_validate(&exif, Tag::DateTimeOriginal);
    let digi = read_and_validate(&exif, Tag::DateTimeDigitized);
    let create = read_and_validate(&exif, Tag::DateTime);
    let gps = read_and_validate(&exif, Tag::GPSDateStamp);

    let selected = [orig, digi, create, gps]
        .into_iter()
        .flatten()
        .reduce(|l, r| if l > r { r } else { l });

    selected.map_or(
        Err(ReadError {
            msg: "No Date Time in file".to_string(),
        }),
        |sdt| {
            Ok(CopyImage {
                source: path,
                date_time: sdt,
            })
        },
    )
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

fn validate_or(
    new_date: Option<SortedDayTime>,
    old_date: Option<SortedDayTime>,
) -> Option<SortedDayTime> {
    if let Some(dt) = &new_date {
        if dt.date_time.year > 0
            && dt.date_time.day > 0
            && dt.date_time.month > 0
            && dt.date_time.day < 32
            && dt.date_time.month < 13
        {
            return new_date;
        }
    }
    old_date
}
