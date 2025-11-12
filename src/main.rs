use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use ini_parser::{IniError, parse_ini};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "CLI для парсингу INI файлів")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Парсить вказаний INI файл та виводить результат
    Parse {
        /// Шлях до .ini файлу
        #[arg(short, long)]
        file: PathBuf,
    },
    /// Відображає інформацію про авторів
    Credits,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { file } => {
            let file_path_str = file.to_string_lossy();
            let content = fs::read_to_string(&file)
                .with_context(|| format!("Не вдалося прочитати файл: {}", file_path_str))?;

            match parse_ini(&content) {
                Ok(data) => {
                    println!("Файл успішно розпарсено: {}", file_path_str);
                    println!("{:#?}", data);
                }
                Err(e @ IniError::ParseError(_)) => {
                    eprintln!("Помилка парсингу файлу:");
                    eprintln!("{}", e);
                }
                Err(e) => {
                    eprintln!("Виникла помилка: {}", e);
                }
            }
        }
        Commands::Credits => {
            println!("INI Parser v0.1.0");
            println!("Створено за допомогою Rust та pest.");
        }
    }

    Ok(())
}
