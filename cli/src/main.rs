use anyhow::{Context, Result};
use clap::Parser;
use oshash::oshash;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "oshash", about = "A tool for hashing files using OSHash algorithm", long_about = None)]
struct Cli {
    #[arg(short, long)]
    bench: bool,
    /// Files to hash (default positional argument)
    #[arg()]
    files: Vec<PathBuf>,
}

static COUNT: u32 = 1000;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let files = cli.files;
    if files.is_empty() {
        return Err(anyhow::anyhow!("No files provided to hash"));
    }

    if cli.bench {
        let start = std::time::Instant::now();
        for i in 0..COUNT {
            process_files(&files, i == COUNT - 1).expect("Failed to process files");
        }

        let duration = start.elapsed();
        println!("Processed {} files 1000x in {:?}", files.len(), duration);
    } else {
        process_files(&files, true)?;
    }

    Ok(())
}
fn process_files(files: &[PathBuf], print: bool) -> Result<()> {
    for file in files {
        let hash = oshash(
            file.as_os_str()
                .to_str()
                .context("could not convert to os_str")?,
        )
        .with_context(|| format!("Failed to hash file: {}", file.display()))?;

        if print {
            println!("{hash} {}", file.display());
        }
    }

    Ok(())
}
