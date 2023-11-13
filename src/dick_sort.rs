use anyhow::Context;
use std::fs;
use std::path::PathBuf;

use exif::DateTime as ExifDateTime;

use crate::shell::{PrintLevel, Shell};
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

pub fn sort(args: Cli, shell: &mut Shell) -> anyhow::Result<()> {
    create_target_dir(&args, shell).with_context(|| {
        format!(
            "Could not create destination dir {}",
            args.destination_dir.display()
        )
    })?;

    // TODO: A generator pattern would work really nicely here.
    //       That way the caller could decide whether to collect or to immediately process a file.
    // dick_sort dir
    let files = file_scanner::scan(
        args.source_dir.clone(),
        shell,
        args.progress,
        args.recursive,
    )
    .context("File scanning failed.")?;

    process::process(&args, files);
    Ok(())
}

fn create_target_dir(args: &Cli, shell: &mut Shell) -> anyhow::Result<()> {
    if args.dry_run || args.destination_dir.exists() {
        return Ok(());
    }

    let dest_dir_str = args.destination_dir.display().to_string();

    shell.println(PrintLevel::Verbose, || {
        format!("Creating Destination dir {}", &dest_dir_str)
    });

    fs::create_dir_all(&args.destination_dir)
        .with_context(|| format!("Could not create destination dir: {}", &dest_dir_str))?;

    shell.println(PrintLevel::Verbose, || format!("Created {}", &dest_dir_str));

    Ok(())
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct SortedDayTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nanosecond: Option<u32>,
    pub offset: Option<i16>,
}

impl From<ExifDateTime> for SortedDayTime {
    fn from(value: ExifDateTime) -> Self {
        let ExifDateTime { year , month, day, hour, minute, second, nanosecond, offset } = value;
        Self { year , month, day, hour, minute, second, nanosecond, offset }
    }
}


#[cfg(test)]
mod tests {
    use exif::DateTime as ExifDateTime;

    use crate::dick_sort::SortedDayTime;

    #[test]
    fn gt_ge_let_le() {
        let younger = ExifDateTime::from_ascii(b"2016:05:04 03:02:01").expect("should be ok");
        let older = ExifDateTime::from_ascii(b"2016:05:04 03:02:00").expect("should be ok");

        let younger = SortedDayTime::from(younger);
        let older = SortedDayTime::from(older);
        assert!(younger > older);
        assert!(younger >= older);
        assert!(older < younger);
        assert!(older <= younger);
    }

    #[test]
    fn eq() {
        let a = ExifDateTime::from_ascii(b"2016:05:04 03:02:00").expect("should be ok");
        let also_a = ExifDateTime::from_ascii(b"2016:05:04 03:02:00").expect("should be ok");

        let b = SortedDayTime::from(a);
        let still_a = SortedDayTime::from(also_a);
        assert_eq!(b, still_a);
    }
}
