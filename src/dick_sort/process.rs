use std::{fs, io};
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fs::remove_dir;
use std::path::PathBuf;

use crate::Cli;
use crate::dick_sort::{CopyImage, ReadError};


use pathdiff::diff_paths;

pub fn process(args: &Cli, mut files: VecDeque<CopyImage>) {
    while !files.is_empty() {
        if args.r#move {
            match move_file(args, &mut files) {
                Ok(true) => {
                    // todo: count moves
                }
                Err(e) => {
                    eprintln!("Failed {:?}", e);
                    copy_and_count(args, &mut files);
                }
                _def => {
                    // todo: count same file
                }
            }
        } else {
            copy_and_count(args, &mut files);
        }
    }
}


fn copy_and_count(args: &Cli, files: &mut VecDeque<CopyImage>) {
    match copy_file(args, files) {
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
            let relative_source = diff_paths(&image.source, &args.source_dir).unwrap();
            let relative_destination = diff_paths(&path, &args.destination_dir).unwrap();
            println!("Would copy from {:?} to {:?}", relative_source, relative_destination);
            Ok(false)
        } else {
            let size = fs::copy(image.source, &path).map_err(|err| ReadError { msg: err.to_string() })?;
            if args.verbose {
                let relative_destination = diff_paths(&path, &args.destination_dir).unwrap();
                println!("Copied {:?} bytes to {:?}", size, relative_destination);
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
            let relative_source = diff_paths(&image.source, &args.source_dir).unwrap();
            let relative_destination = diff_paths(&path, &args.destination_dir).unwrap();
            println!("Would move from {:?} to {:?}", relative_source, relative_destination);
            Ok(false)
        } else {
            fs::rename(&image.source, &path).map_err(|err| ReadError { msg: err.to_string() })?;
            let size = fs::metadata(&path).unwrap().len();
            if args.verbose {
                let relative_destination = diff_paths(&path, &args.destination_dir).unwrap();
                println!("Moved {:?} bytes to {:?}", size, relative_destination);
            }
            if args.clean {
                // If we can't delete it's no reason to stop moving
                let _ = clean_empty_to_root(args, &image.source.parent().expect("A file should have a parent").to_path_buf(), &args.source_dir);
            }
            Ok(true)
        };
    }
    Ok(false)
}

fn clean_empty_to_root(args: &Cli, current: &PathBuf, root: &PathBuf) -> Result<(), ReadError> {

    // while we haven't reached the root dir, we process parents
    let recurse = current != root;

    // make sure that the root dir is actually root of the current

    if !current.starts_with(root) {
        return Err(ReadError { msg: "Given current path is not sub dir of given root".to_string() });
    }

    return match remove_dir(current) {
        Ok(_) => {
            if args.verbose {
                println!("Deleted empty dir {}", current.display())
            }
            if recurse {
                return match current.parent() {
                    Some(path) => {
                        clean_empty_to_root(args, &path.to_path_buf(), root)
                    }
                    _ => {
                        Ok(())
                    }
                };
            }
            Ok(())
        }
        Err(ref err) if err.kind() == io::ErrorKind::NotFound => {
            Ok(())
        }
        Err(err) => {
            Err(ReadError { msg: err.to_string() })
        }
    };
}

fn build_and_create_path(args: &Cli, files: &mut VecDeque<CopyImage>) -> Result<(PathBuf, CopyImage), ReadError> {
    let destination = args.destination_dir.to_str().ok_or(ReadError { msg: "destination dir has no string".to_string() })?;
    let image = files.pop_front().ok_or(ReadError { msg: "No more elements in queue".to_string() })?;
    let name = image.source.file_name().ok_or(ReadError { msg: "File has no filename".to_string() })?;

    let path = create_sub_path(args, destination, &image, name)?;
    Ok((path, image))
}

fn create_sub_path(args: &Cli, dest: &str, image: &CopyImage, file_name: &OsStr) -> Result<PathBuf, ReadError> {

    // replace placeholders with exif value
    let mut relative_path = args.format.replace("[YEAR]", format!("{:04}", image.date_time.date_time.year).as_str())
        .replace("[MONTH]", format!("{:02}", image.date_time.date_time.month).as_str())
        .replace("[DAY]", format!("{:02}", image.date_time.date_time.day).as_str());

    // add file name
    relative_path.push_str(file_name.to_str().unwrap());
    // make absolute
    let mut absolut = dest.to_string();
    absolut.push_str(relative_path.as_str());

    // create PathBuf
    let absolute_path = PathBuf::from(absolut);
    if !args.dry_run {
        // create parent dirs
        fs::create_dir_all(absolute_path.parent().expect("The file should have a parent dir")).map_err(|err| ReadError { msg: err.to_string() })?;
    }

    Ok(absolute_path)
}
