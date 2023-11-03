use std::cmp::Ordering;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

use exif::DateTime;

use crate::Cli;

mod file_scanner;
mod process;

#[derive(Debug)]
pub struct CopyImage {
    pub source: PathBuf,
    pub date_time: SortedDayTime,
}


#[derive(Debug)]
pub struct ReadError {
    pub msg: String,
}

pub fn sort(args: Cli) -> Result<(), ReadError> {
    let created_dir = create_target_dir(&args);
    if created_dir.is_err() {
        return Err(ReadError { msg: format!("Could not create destination dir {} {}", &args.destination_dir.display(), created_dir.err().unwrap().msg) });
    }

    let mut unprocessed_directories: VecDeque<PathBuf> = VecDeque::new();

    // add source dir in the queue
    unprocessed_directories.push_back(args.source_dir.clone());

    // dick_sort dir
    let files = file_scanner::scan(&args, unprocessed_directories);

    process::process(&args, files);
    Ok(())
}

fn create_target_dir(args: &Cli) -> Result<(), ReadError> {
    if !args.dry_run && !args.destination_dir.exists() {
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

#[derive(Debug)]
pub struct SortedDayTime {
    pub date_time: DateTime,
}

impl SortedDayTime {
    pub fn new(dt: DateTime) -> Self {
        SortedDayTime { date_time: dt }
    }
}

impl PartialEq<SortedDayTime> for SortedDayTime {
    fn eq(&self, other: &SortedDayTime) -> bool {
        if self.date_time.year != other.date_time.year {
            return false;
        }
        if self.date_time.month != other.date_time.month {
            return false;
        }
        // todo: offsets?

        if self.date_time.day != other.date_time.day {
            return false;
        }
        if self.date_time.hour != other.date_time.hour {
            return false;
        }
        if self.date_time.minute != other.date_time.minute {
            return false;
        }
        if self.date_time.second != other.date_time.second {
            return false;
        }
        true
    }
}

impl PartialOrd<SortedDayTime> for SortedDayTime {
    fn partial_cmp(&self, other: &SortedDayTime) -> Option<Ordering> {
        if self.date_time.year > other.date_time.year {
            return Some(Greater);
        }
        if self.date_time.year < other.date_time.year {
            return Some(Less);
        }
        if self.date_time.month > other.date_time.month {
            return Some(Greater);
        }
        if self.date_time.month < other.date_time.month {
            return Some(Less);
        }
        // todo: offsets?

        if self.date_time.day > other.date_time.day {
            return Some(Greater);
        }
        if self.date_time.day < other.date_time.day {
            return Some(Less);
        }
        if self.date_time.hour > other.date_time.hour {
            return Some(Greater);
        }
        if self.date_time.hour < other.date_time.hour {
            return Some(Less);
        }
        if self.date_time.minute > other.date_time.minute {
            return Some(Greater);
        }
        if self.date_time.minute < other.date_time.minute {
            return Some(Less);
        }
        if self.date_time.second > other.date_time.second {
            return Some(Greater);
        }
        if self.date_time.second < other.date_time.second
        {
            return Some(Less);
        }
        Some(Equal)
    }

    fn lt(&self, other: &SortedDayTime) -> bool {
        if let Some(order) = self.partial_cmp(other) {
            if order == Less {
                return true;
            }
        }
        false
    }

    fn le(&self, other: &SortedDayTime) -> bool {
        if let Some(order) = self.partial_cmp(other) {
            if order == Less || order == Equal {
                return true;
            }
        }
        false
    }

    fn gt(&self, other: &SortedDayTime) -> bool {
        !self.le(other)
    }

    fn ge(&self, other: &SortedDayTime) -> bool {
        !self.lt(other)
    }
}

#[cfg(test)]
mod tests {
    use exif::DateTime;

    use crate::dick_sort::SortedDayTime;

    #[test]
    fn gt_ge_let_le() {
        let younger = DateTime::from_ascii(b"2016:05:04 03:02:01").expect("should be ok");
        let older = DateTime::from_ascii(b"2016:05:04 03:02:00").expect("should be ok");

        let younger = SortedDayTime::new(younger);
        let older = SortedDayTime::new(older);
        assert!(younger > older);
        assert!(younger >= older);
        assert!(older < younger);
        assert!(older <= younger);
    }

    #[test]
    fn eq() {
        let a = DateTime::from_ascii(b"2016:05:04 03:02:00").expect("should be ok");
        let also_a = DateTime::from_ascii(b"2016:05:04 03:02:00").expect("should be ok");

        let b = SortedDayTime::new(a);
        let still_a = SortedDayTime::new(also_a);
        assert_eq!(b, still_a);
    }
}