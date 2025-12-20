use std::env;

#[cfg(not(feature = "tokio"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use oshash::oshash;
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];

    match oshash(file_path) {
        Ok(hash) => {
            println!("OSHash of '{}': {}", file_path, hash);
        }
        Err(e) => {
            eprintln!("Error hashing file: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use oshash::oshash_async;
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];

    match oshash_async(file_path).await {
        Ok(hash) => {
            println!("OSHash of '{}': {}", file_path, hash);
        }
        Err(e) => {
            eprintln!("Error hashing file: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
