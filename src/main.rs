#![deny(clippy::style)]
#![deny(clippy::perf)]

mod anonymizer;
mod parser;
use clap::{Parser as ClapParser, Subcommand};
use std::fs;
use std::io::{self, Write};

#[cfg(test)]
mod tests;

#[derive(ClapParser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Распарсить выписку
    Parse {
        /// Путь к файлу
        #[arg()]
        file: String,
    },
    /// Анонимизировать выписку
    Anon {
        /// Путь к файлу
        #[arg()]
        file: String,
    },
}

fn main() -> Result<(), io::Error> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Parse { file } => {
            let input = fs::read(&file)?;
            let parsed = parser::Parser::default()
                .parse(&input)
                .map_err(|err| io::Error::other(format!("{err:?}")))?;
            println!("{parsed:#?}");
        }
        Commands::Anon { file } => {
            let input = fs::read(&file)?;
            let (text, encoding) = decode_any_encoding(&input);
            let anonymized = anonymizer::anonymize_str(&text);
            let output_bytes = match encoding {
                "cp1251" => encoding_rs::WINDOWS_1251.encode(&anonymized).0.into_owned(),
                "koi8-r" => encoding_rs::KOI8_R.encode(&anonymized).0.into_owned(),
                _ => anonymized.into_bytes(),
            };
            io::stdout().write_all(&output_bytes)?;
        }
    }
    Ok(())
}

fn decode_any_encoding(input: &[u8]) -> (String, &'static str) {
    // Try UTF-8 first
    if let Ok(s) = std::str::from_utf8(input) {
        return (s.to_string(), "utf-8");
    }
    // Try cp1251
    let (cow, _, _) = encoding_rs::WINDOWS_1251.decode(input);
    if !cow.is_empty() && !cow.chars().all(|c| c == '\u{fffd}') {
        return (cow.into_owned(), "cp1251");
    }
    // Try koi8-r
    let (cow, _, _) = encoding_rs::KOI8_R.decode(input);
    if !cow.is_empty() && !cow.chars().all(|c| c == '\u{fffd}') {
        return (cow.into_owned(), "koi8-r");
    }
    // Fallback: lossy utf-8
    (String::from_utf8_lossy(input).into_owned(), "unknown")
}
