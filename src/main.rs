#![allow(unused)]

use clap::Parser;

/// Sorts pics from one directory into another one
#[derive(Parser,Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The path from where the pics will be read
    #[clap(parse(from_os_str))]
    source_dir: std::path::PathBuf,

    /// The path to where the pics will be written. This must not be the source_dir!
    #[clap(parse(from_os_str))]
    destination_dir: std::path::PathBuf,

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
    let args = Cli::parse();
    println!("{:#?}", args);
}
