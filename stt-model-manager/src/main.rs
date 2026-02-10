use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available models
    List,
    /// Download a model
    Download {
        /// Name of the model to download (e.g., "base", "small", "medium")
        name: String,
        /// Force re-download
        #[arg(short, long)]
        force: bool,
        /// Download to global system directory (/usr/share/stt-assistant/models)
        #[arg(short, long)]
        global: bool,
    },
    /// Show the storage paths
    Path,
}

struct ModelInfo {
    name: &'static str,
    url: &'static str,
    description: &'static str,
}

const MODELS: &[ModelInfo] = &[
    ModelInfo {
        name: "tiny",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin",
        description: "Tiny model (lowest accuracy)",
    },
    ModelInfo {
        name: "base",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin",
        description: "Base model (standard balance)",
    },
    ModelInfo {
        name: "small",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
        description: "Small model",
    },
    ModelInfo {
        name: "medium",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin",
        description: "Medium model",
    },
    ModelInfo {
        name: "large-v3",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin",
        description: "Large v3 model (highest accuracy)",
    },
];

fn get_local_models_dir() -> Result<PathBuf> {
    let mut path = dirs::data_local_dir().context("Could not find local data directory")?;
    path.push("stt-assistant");
    path.push("models");
    Ok(path)
}

fn get_global_models_dir() -> PathBuf {
    PathBuf::from("/usr/share/stt-assistant/models")
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::List => {
            let local_dir = get_local_models_dir()?;
            let global_dir = get_global_models_dir();

            println!("Available models from HuggingFace (ggerganov/whisper.cpp):");
            println!(
                "{:<12} {:<40} {:<10} {:<10}",
                "NAME", "DESCRIPTION", "LOCAL", "GLOBAL"
            );
            println!("{:-<12} {:-<40} {:-<10} {:-<10}", "", "", "", "");
            for model in MODELS {
                let local_path = local_dir.join(format!("ggml-{}.bin", model.name));
                let global_path = global_dir.join(format!("ggml-{}.bin", model.name));

                let local_status = if local_path.exists() { "YES" } else { "-" };
                let global_status = if global_path.exists() { "YES" } else { "-" };

                println!(
                    "{:<12} {:<40} {:<10} {:<10}",
                    model.name, model.description, local_status, global_status
                );
            }

            println!("\nNote: stt-daemon prioritizes LOCAL models over GLOBAL ones.");

            println!("Example Download URL: {}", MODELS[1].url);
        }
        Commands::Path => {
            println!("Local:  {}", get_local_models_dir()?.display());
            println!("Global: {}", get_global_models_dir().display());
        }
        Commands::Download {
            name,
            force,
            global,
        } => {
            let model = MODELS.iter().find(|m| m.name == name).ok_or_else(|| {
                anyhow::anyhow!(
                    "Model '{}' not found. Use 'list' to see available models.",
                    name
                )
            })?;

            let target_dir = if global {
                get_global_models_dir()
            } else {
                get_local_models_dir()?
            };

            if !target_dir.exists() {
                std::fs::create_dir_all(&target_dir).with_context(|| {
                    format!(
                        "Failed to create directory {}. Check permissions (use sudo for --global).",
                        target_dir.display()
                    )
                })?;
            }

            let file_name = format!("ggml-{}.bin", model.name);
            let dest_path = target_dir.join(&file_name);

            if dest_path.exists() && !force {
                println!(
                    "Model '{}' already exists at {}. Use --force to overwrite.",
                    name,
                    dest_path.display()
                );
                return Ok(());
            }

            println!("Downloading {} to {}...", model.name, dest_path.display());
            download_file(model.url, &dest_path).await?;
            println!("Download complete.");
        }
    }

    Ok(())
}

async fn download_file(url: &str, path: &Path) -> Result<()> {
    let res = reqwest::get(url)
        .await
        .context("Failed to initiate request")?;
    let total_size = res.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
        .progress_chars("#>-"));

    let mut file = File::create(path).context("Failed to create file")?;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.context("Error while downloading chunk")?;
        file.write_all(&chunk)
            .context("Error while writing to file")?;
        pb.inc(chunk.len() as u64);
    }

    pb.finish_with_message("Downloaded");
    Ok(())
}
