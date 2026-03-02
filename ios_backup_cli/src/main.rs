use anyhow::Result;
use clap::{Parser, Subcommand};
use ios_backup_core::mbdb;
use ios_backup_core::{FileRecord, gather_extensions, infer::FileHeaderReader, parse};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug, Parser)]
#[command(name = "iOS Backup Explorer")]
#[command(about = "Decode Manifest.mbdb and extract files from iOS backups")]
struct Args {
    #[command(subcommand)]
    command: Command,
    #[arg(value_name = "BACKUP_DIR")]
    backup_dir: PathBuf,
}

#[derive(Debug, Subcommand)]
enum Command {
    ListExtensions,
    Extract {
        #[arg(value_name = "DEST_DIR")]
        output: PathBuf,
        #[arg(
            short,
            long,
            value_name = "EXTENSIONS",
            conflicts_with = "include_extensions"
        )]
        exclude_extensions: Option<Vec<String>>,
        #[arg(short, long, value_name = "EXTENSIONS")]
        include_extensions: Option<Vec<String>>,
    },
}

struct FsFileHeaderReader<'a> {
    backup_dir: &'a PathBuf,
}

impl FileHeaderReader for FsFileHeaderReader<'_> {
    async fn read_header(&self, file_name: &str) -> Vec<u8> {
        let mut file = File::open(self.backup_dir.join(file_name)).await.unwrap();
        let limit = (file.metadata().await.unwrap().len()).min(8192u64) as usize;
        let mut buf = vec![0; limit];
        file.read_exact(&mut buf).await.unwrap();
        buf
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let files = parse(
        &mut File::open(&args.backup_dir.join("Manifest.mbdb")).await?,
        &FsFileHeaderReader {
            backup_dir: &args.backup_dir,
        },
    )
    .await?;

    match args.command {
        Command::ListExtensions => print_extensions(files.as_slice()),
        Command::Extract {
            output,
            exclude_extensions,
            include_extensions,
        } => {
            if output.exists() {
                fs::remove_dir_all(&output)?;
            }
            fs::create_dir_all(&output)?;

            let mut copied = 0;
            let mut excluded = 0;

            for file in &files {
                let should_skip = include_extensions.as_ref().map_or_else(
                    || {
                        exclude_extensions.as_ref().is_some_and(|excluded| {
                            excluded.contains(&file.inferred_extension.extension)
                        })
                    },
                    |included| !included.contains(&file.inferred_extension.extension),
                );

                if should_skip {
                    excluded += 1;
                    continue;
                }

                copy_file(&file.mbdb, &args.backup_dir, &output)?;
                copied += 1;
            }

            println!("Total files detected: {}", files.len());
            println!("Successfully copied: {copied}");
            if excluded > 0 {
                println!("Excluded: {excluded}");
            }
        }
    }

    Ok(())
}

fn copy_file(file: &mbdb::FileRecord, backup_dir: &Path, output_dir: &Path) -> Result<()> {
    let source_path = backup_dir.join(&file.file_id);
    let dest_path = output_dir.join(&file.domain).join(&file.path);
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source_path, &dest_path)?;
    Ok(())
}

fn print_extensions(files: &[FileRecord]) {
    gather_extensions(files)
        .into_iter()
        .for_each(|(ext_type, (total_count, extensions))| {
            println!("{ext_type} ({total_count} files)");
            for (ext, count) in extensions {
                println!("  - .{ext} ({count} files)");
            }
        });
}
