use std::fs::File;
use chrono::Datelike;
use xwords::{fill::Fill, trie::Trie};

use clap::{App, Arg};
use xwords::{crossword::Crossword, fill::filler::Filler};
use inflector::Inflector;
use xwords::across::AcrossFileFormat;

fn main() -> Result<(), String> {
    let matches = App::new("xwords")
        .arg(Arg::from_usage("-i, --input <FILE> 'Input crossword file location.'"))
        .arg(Arg::from_usage("[random] -r, --random 'Randomize word fill. Default is false.'"))
        .arg(Arg::from_usage("[max-time] -m, --max-time <SECONDS> 'Maximum number of seconds to process. Default is 120s (2 minutes).'"))
        .arg(Arg::from_usage("[words] -w, --words <WORDS_FILE_NAME> 'File name from /words without extension to use for filling. Default is `en`.'"))
        .arg(Arg::from_usage("[format] -f, --format <FORMAT> 'Output format. Can be `grid` for simple grid or `across` for Across Puzzle V2 text. Default is `grid`.'"))
        .arg(Arg::from_usage("[title] -t, --title <TITLE> 'Puzzle title for across output. Defaults to title case file name.'"))
        .arg(Arg::from_usage("[author] -a, --author <AUTHOR> 'Author name across output. Defaults to `xwords-rs`.'"))
        .arg(Arg::from_usage("[copyright] -c, --copyright <COPYRIGHT> 'Copyright text for across output. Defaults to `<YEAR> Public domain.`'"))
        .arg(Arg::from_usage("[log] -l, --log 'Prints intermediate progress information to stderr. Default is false.'"))
        .arg(Arg::from_usage("[profile] -p, --profile 'Profile the program. Default is false.'"))
        .get_matches();

    let input_file_name = matches.value_of("input").expect("input not included");

    let input = Crossword::parse_from_file(input_file_name)
        .expect("Failed to parse crossword from file");

    let random = matches.is_present("random");

    if matches.is_present("profile") {
        let guard = pprof::ProfilerGuard::new(100).unwrap();
        std::thread::spawn(move || loop {
            if let Ok(report) = guard.report().build() {
                let file = File::create("flamegraph.svg").unwrap();
                report.flamegraph(file).unwrap();
            }
            std::thread::sleep(std::time::Duration::from_secs(5))
        });
    }

    let words = matches.value_of("words").unwrap_or("en");
    
    let max_time_seconds = matches.value_of("max-time")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(120); // Default to 120 seconds (2 minutes)
    
    let log = matches.is_present("log");
    
    let trie = Trie::load(words).expect("Failed to load trie");
    let crossword = Filler::new(&trie, random, max_time_seconds, log).fill(&input);

    match crossword {
        Ok(crossword) => {
            let format = matches.value_of("format");
            let format = match format {
                Some("across") => Format::Across,
                Some("grid") => Format::Grid,
                None => Format::Grid,
                _ => Err(format!("Invalid format: {}", format.unwrap()))?,
            };

            match format {
                Format::Across => {
                    let title = matches.value_of("title")
                        .unwrap_or(&*input_file_name.to_title_case())
                        .to_string();
                    let author = matches.value_of("author")
                        .unwrap_or("xwords-rs")
                        .to_string();
                    let copyright = matches
                        .value_of("copyright")
                        .unwrap_or(&*format!("{} Public domain", chrono::Local::now().year()))
                        .to_string();
                    let across = AcrossFileFormat::new(crossword, title, author, copyright);
                    println!("{}", across);
                }
                Format::Grid => {
                    println!("{}", crossword);
                }
            }
        }
        Err(s) => return Err(String::from(format!("Failed to fill crossword: {}", s))),
    }
    Ok(())
}


/// The format of the output.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Format {
    Across, // Across Puzzle V2 file format
    Grid, // Simple Grid format
}
