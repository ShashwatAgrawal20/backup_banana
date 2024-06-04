use anyhow::Result;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use walkdir::WalkDir;

async fn copy_file(src: PathBuf, dest: PathBuf) -> Result<()> {
    async_fs::copy(src, dest).await?;
    Ok(())
}

async fn async_main() -> Result<()> {
    let src_dir = get_full_path_from_user_input("Enter src directory path: ")?;
    let dest_dir = get_full_path_from_user_input("Enter dest directory path: ")?;

    let mut tasks = Vec::new();

    for entry in WalkDir::new(&src_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
    {
        let src_path = entry.clone().path().to_owned();
        let relative_path = src_path.strip_prefix(&src_dir)?;
        let dest_path = PathBuf::from(&dest_dir).join(relative_path);

        if src_path.is_file() {
            let task = async move {
                async_fs::create_dir_all(dest_path.parent().unwrap()).await?;
                let _ = copy_file(src_path.into(), dest_path.into()).await;
                Ok::<(), io::Error>(())
            };
            tasks.push(task);
        }
    }
    futures::future::try_join_all(tasks).await?;
    Ok(())
}

fn get_full_path_from_user_input(prompt: &str) -> Result<PathBuf, io::Error> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let path = Path::new(&input.trim()).to_path_buf();

    if path.exists() {
        if let Ok(canonical_path) = path.canonicalize() {
            return Ok(canonical_path);
        }
    } else {
        fs::create_dir_all(&path)?;
        if let Ok(canonical_path) = path.canonicalize() {
            return Ok(canonical_path);
        }
    }
    Ok(path)
}

#[tokio::main]
async fn main() {
    if let Err(err) = async_main().await {
        eprintln!("Error: {}", err);
    }
}
