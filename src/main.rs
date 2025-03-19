#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

/**
 * grep-rs: grep but in Rust
 * Copyright (C) 2025 DarkCeptor44
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
mod errors;

use clap::Parser;
use colored::{Color, Colorize};
use errors::{GrepError, Result};
use std::{
    fs::File,
    io::{BufRead, BufReader, stdin},
    path::PathBuf,
    process::exit,
    str::FromStr,
};

#[derive(Parser, Debug)]
#[command(author,version,about,long_about=None)]
struct App {
    #[arg(help = "The pattern to search for")]
    pattern: String,

    #[arg(help = "The path to the file")]
    path: Option<String>,

    #[arg(
        short = 'C',
        long,
        help = "Use case-sensitive search",
        default_value_t = false
    )]
    case_sensitive: bool,

    #[arg(short, long, help = "Show line numbers", default_value_t = false)]
    line_numbers: bool,

    #[arg(
        short,
        long,
        help = "The color to use for highlighting (off for no color)",
        default_value = "blue"
    )]
    color: String,
}

fn main() {
    if let Err(e) = App::run() {
        eprintln!("{}", e.to_string().red());
        exit(1);
    }
}

impl App {
    fn run() -> Result<()> {
        let args = App::parse();
        let color = Color::from_str(&args.color).unwrap_or(Color::Blue);

        if args.pattern.trim().is_empty() {
            return Err(GrepError::Io("Pattern cannot be empty".into()));
        }

        let reader: Box<dyn BufRead> = if let Some(path_str) = args.path {
            let path = PathBuf::from(&path_str);

            if !path.is_file() {
                return Err(GrepError::Io(format!(
                    "{} is not a file",
                    path.display().to_string().bold()
                )));
            }

            let file = File::open(path)?;
            Box::new(BufReader::new(file))
        } else {
            let stdin = stdin();
            Box::new(BufReader::new(stdin))
        };

        let pattern = &args.pattern;
        let pattern_lower = pattern.to_lowercase();

        for (line_number, line_result) in (1_u64..).zip(reader.lines()) {
            let line = line_result?;
            let operation = if args.case_sensitive {
                line.contains(pattern)
            } else {
                line.to_lowercase().contains(&pattern_lower)
            };

            if operation {
                println!(
                    "{}{}",
                    if args.line_numbers {
                        format!("{line_number}:  ").bold()
                    } else {
                        String::new().normal()
                    },
                    highlight_text(&line, pattern, color)
                );
            }
        }

        Ok(())
    }
}

/// Highlights the given text in the given color
fn highlight_text(text: &str, to_highlight: &str, color: Color) -> String {
    let index = text
        .to_lowercase()
        .find(&to_highlight.to_lowercase())
        .unwrap_or(0);
    format!(
        "{}{}{}",
        text[..index].normal(),
        text[index..index + to_highlight.len()].bold().color(color),
        text[index + to_highlight.len()..].normal()
    )
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{File, remove_file},
        io::Write,
        process::{Command, Stdio},
    };

    const BIN_PATH: &str = "target/debug/grep-rs";
    const TEST_FILE_NAME: &str = "test_file_";

    #[test]
    fn test_find_pattern_in_file() -> Result<(), Box<dyn std::error::Error>> {
        let file_name = &format!("{TEST_FILE_NAME}1.txt");
        let mut file = File::create(file_name)?;
        writeln!(file, "This line contains the pattern.")?;
        writeln!(file, "Another line without it.")?;

        let mut cmd = Command::new(BIN_PATH);
        cmd.arg("pattern").arg(file_name);
        let output = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).output()?;
        assert!(output.status.success());

        let stdout = String::from_utf8(output.stdout)?;
        assert!(stdout.contains("This line contains the pattern."));

        remove_file(file_name)?;
        Ok(())
    }

    #[test]
    fn test_pattern_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let file_name = &format!("{TEST_FILE_NAME}2.txt");
        let mut file = File::create(file_name)?;
        writeln!(file, "This line does not match.")?;

        let mut cmd = Command::new(BIN_PATH);
        cmd.arg("This line does NOT match.")
            .arg(file_name)
            .arg("-C");
        let output = cmd.stdout(Stdio::piped()).output()?;

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout)?;
        assert!(!stdout.contains("This line does NOT match."));

        remove_file(file_name)?;
        Ok(())
    }

    #[test]
    fn test_empty_pattern() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::new(BIN_PATH);
        cmd.arg("");
        let output = cmd.stderr(Stdio::piped()).output()?;

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr)?;
        assert!(stderr.contains("grep-rs: Pattern cannot be empty"));

        Ok(())
    }
}
