use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(author, version, about = "Telora Model Manager - Download and manage Whisper models", long_about = None)]
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
        name: Option<String>,
        /// Force re-download
        #[arg(short, long)]
        force: bool,
        /// Download to global system directory (/usr/share/telora/models)
        #[arg(short, long)]
        global: bool,
        /// Custom URL to download from
        #[arg(short, long)]
        url: Option<String>,
        /// Custom output filename
        #[arg(short, long)]
        out: Option<String>,
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
    path.push("telora");
    path.push("models");
    Ok(path)
}

fn get_global_models_dir() -> PathBuf {
    PathBuf::from("/usr/share/telora/models")
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::List => {
            let local_dir = get_local_models_dir()?;
            let global_dir = get_global_models_dir();

            println!("Available models in https://huggingface.co/ggerganov/whisper.cpp:");
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

            println!("\nNote: telora-daemon prioritizes LOCAL models over GLOBAL ones.");
        }
        Commands::Path => {
            println!("Local:  {}", get_local_models_dir()?.display());
            println!("Global: {}", get_global_models_dir().display());
        }
        Commands::Download {
            name,
            force,
            global,
            url,
            out,
        } => {
            let (download_url, model_identifier) = if let Some(custom_url) = url.clone() {
                (custom_url, name.clone())
            } else {
                let name = name
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("Model name or --url is required."))?;
                if let Some(model) = MODELS.iter().find(|m| m.name == name) {
                    println!("Download from https://huggingface.co/ggerganov/whisper.cpp");
                    (model.url.to_string(), Some(model.name.to_string()))
                } else {
                    println!("Download from https://huggingface.co/ggerganov/whisper.cpp");
                    let constructed_url = format!(
                        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
                        name
                    );
                    (constructed_url, Some(name))
                }
            };

            let file_name = if let Some(output_name) = out {
                output_name
            } else if url.is_some() {
                // If URL is provided, default to the filename in the URL
                Path::new(&download_url)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Could not determine filename from URL. Use --out to specify one."
                        )
                    })?
            } else {
                // Default HuggingFace pattern
                format!(
                    "ggml-{}.bin",
                    model_identifier.expect("Model identifier should be present if URL is not")
                )
            };

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

            let dest_path = target_dir.join(&file_name);

            if dest_path.exists() && !force {
                println!(
                    "File '{}' already exists at {}. Use --force to overwrite.",
                    file_name,
                    dest_path.display()
                );
                return Ok(());
            }

            println!("Downloading to {}...", dest_path.display());
            download_file(&download_url, &dest_path).await?;
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
