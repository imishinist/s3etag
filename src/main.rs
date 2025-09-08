use std::fs::File;
use std::path::PathBuf;

use clap::Parser;
use s3etag::Context;

#[derive(Parser)]
#[command(name = "s3etag")]
#[command(about = "Calculate S3 ETag for multipart uploads")]
struct Cli {
    /// Chunk size in MB
    #[arg(short, long, default_value_t = 8)]
    chunk_size: u64,

    /// File path
    file: PathBuf,

    /// Expected ETag to verify against
    #[arg(short, long)]
    etag: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if !cli.file.exists() {
        eprintln!("file not found: {}", cli.file.display());
        std::process::exit(2);
    }

    let mut file = File::open(cli.file)?;
    let chunk_size_bytes = cli.chunk_size * 1024 * 1024;

    let mut context = Context::with_chunk_size(chunk_size_bytes as usize);
    std::io::copy(&mut file, &mut context)?;

    let digest = context.finalize();
    let hash = format!("{digest:x}");

    match cli.etag {
        Some(ref expected) => {
            let trimmed = expected.trim_matches('"');
            if hash == trimmed {
                println!("TRUE");
                return Ok(());
            } else {
                println!("FALSE");
                std::process::exit(1);
            }
        }
        _ => println!("{digest:x}"),
    }
    Ok(())
}
