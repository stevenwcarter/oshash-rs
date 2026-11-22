use anyhow::{Context, Result};
use clap::Parser;
use oshash::oshash;
use std::io::{self, BufWriter, Write};
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

    // One locked, buffered writer for the whole run instead of a lock + flush per line.
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    if cli.bench {
        let start = std::time::Instant::now();
        for i in 0..COUNT {
            process_files(&files, &mut out, i == COUNT - 1).expect("Failed to process files");
        }

        let duration = start.elapsed();
        writeln!(
            out,
            "Processed {} files 1000x in {:?}",
            files.len(),
            duration
        )?;
    } else {
        process_files(&files, &mut out, true)?;
    }

    // Explicit so that write errors surface; on the early returns above `BufWriter`'s drop
    // still flushes the lines for files hashed before the failure.
    out.flush()?;

    Ok(())
}
fn process_files(files: &[PathBuf], out: &mut impl Write, print: bool) -> Result<()> {
    for file in files {
        let hash = oshash(
            file.as_os_str()
                .to_str()
                .context("could not convert to os_str")?,
        )
        .with_context(|| format!("Failed to hash file: {}", file.display()))?;

        if print {
            writeln!(out, "{hash} {}", file.display())?;
        }
    }

    Ok(())
}
