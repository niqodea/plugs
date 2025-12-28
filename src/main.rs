use std::env::{current_dir, var};
use std::fmt::Write;
use std::fs::{copy, create_dir, read_link, remove_dir_all, remove_file, symlink_metadata, File};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

use clap::{Parser, Subcommand};
use walkdir::WalkDir;

const SOCKET_PREFIX: &str = "._.";
const SAMPLE_PREFIX: &str = ".sample.";

fn plug_name(plug_path: &Path) -> Result<&str, String> {
    plug_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("invalid path".to_string())
}

fn socket_path(plug_path: &Path) -> Result<PathBuf, String> {
    let plug_name = plug_name(plug_path)?;
    Ok(plug_path.with_file_name(format!("{SOCKET_PREFIX}{plug_name}")))
}

fn sample_path(plug_path: &Path) -> Result<PathBuf, String> {
    let plug_name = plug_name(plug_path)?;
    Ok(plug_path.with_file_name(format!("{SAMPLE_PREFIX}{plug_name}")))
}

fn validate(plug_path: &Path) -> Result<(), String> {
    if symlink_metadata(plug_path).is_err() {
        return Err(format!("nothing found at {}", plug_path.display()));
    }

    if !plug_path.is_symlink() {
        return Err(format!("not a symlink at {}", plug_path.display()));
    }

    let plug_name = plug_name(plug_path)?;

    let plug_pointer_path =
        read_link(plug_path).map_err(|e| format!("cannot read symlink: {e}"))?;

    let expected_plug_pointer_path = PathBuf::from(format!("{SOCKET_PREFIX}{plug_name}"));

    if plug_pointer_path != expected_plug_pointer_path {
        return Err(format!(
            "symlink should point to {}",
            expected_plug_pointer_path.display()
        ));
    }

    Ok(())
}

#[derive(Parser)]
#[command(
    name = "plug",
    version = "0.1.0",
    about = "Manage plug symlinks for explicit file dependencies"
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "List plugs and their status")]
    Status {
        #[arg(default_value = ".", help = "Directory to search for plugs")]
        root: PathBuf,
    },
    #[command(about = "Create new plug")]
    Create {
        #[arg(help = "Plug path")]
        plug: PathBuf,
    },
    #[command(about = "Connect plug to existing target")]
    Connect {
        #[arg(help = "Plug path")]
        plug: PathBuf,
        #[arg(long, help = "Target path to connect to")]
        to: PathBuf,
        #[arg(long, help = "Treat the path as relative")]
        relative: bool,
    },
    #[command(about = "Disconnect plug")]
    Disconnect {
        #[arg(help = "Plug path")]
        plug: PathBuf,
        #[arg(long, help = "Allow deleting files and directories")]
        delete: bool,
    },
    #[command(about = "Delete plug")]
    Delete {
        #[arg(help = "Plug path")]
        plug: PathBuf,
    },
    #[command(about = "Connect plug to new directory")]
    ConnectNewDir {
        #[arg(help = "Plug path")]
        plug: PathBuf,
    },
    #[command(about = "Connect plug to new file")]
    ConnectNewFile {
        #[arg(help = "Plug path")]
        plug: PathBuf,
        #[arg(long, help = "Copy from sample file")]
        from_sample: bool,
        #[arg(long, help = "Open file with $EDITOR")]
        edit: bool,
    },
    #[command(about = "Write sample file for plug")]
    WriteSampleFile {
        #[arg(help = "Plug path")]
        plug: PathBuf,
    },
}

fn main() {
    let args = Args::parse();
    let result = match args.command {
        Commands::Status { root } => status(&root),
        Commands::Create { plug } => create(&plug),
        Commands::Connect { plug, to, relative } => connect(&plug, &to, relative),
        Commands::Disconnect { plug, delete } => disconnect(&plug, delete),
        Commands::Delete { plug } => delete(&plug),
        Commands::ConnectNewDir { plug } => connect_new_dir(&plug),
        Commands::ConnectNewFile {
            plug,
            from_sample,
            edit,
        } => connect_new_file(&plug, from_sample, edit),
        Commands::WriteSampleFile { plug } => write_sample_file(&plug),
    };

    match result {
        Ok(stdout) => {
            println!("{stdout}");
        }
        Err(stderr) => {
            eprintln!("{stderr}");
            exit(1);
        }
    }
}

fn status(root: &Path) -> Result<String, String> {
    let mut output = String::new();

    for entry in WalkDir::new(root) {
        let entry = entry.map_err(|e| format!("walk failed: {e}"))?;

        let entry_path = entry
            .path()
            .strip_prefix(root)
            .map_err(|e| format!("strip prefix failed: {e}"))?;

        if validate(entry_path).is_err() {
            continue;
        }

        let plug_path = entry_path;
        let socket_path = socket_path(plug_path)?;

        let target_path = if socket_path.is_symlink() {
            Some(read_link(&socket_path).map_err(|e| format!("cannot read socket symlink: {e}"))?)
        } else {
            None
        };

        let status = if let Some(path) = &target_path {
            if path.exists() {
                'S'
            } else {
                'X'
            }
        } else if socket_path.is_dir() {
            'D'
        } else if socket_path.is_file() {
            'F'
        } else {
            ' '
        };

        writeln!(output, "{} <- {}", status, plug_path.display())
            .map_err(|e| format!("write failed: {e}"))?;

        if let Some(path) = &target_path {
            writeln!(output, " `-> {}", path.display())
                .map_err(|e| format!("write failed: {e}"))?;
        }
    }

    if output.is_empty() {
        output.push_str("No plugs found.\n");
    }

    Ok(output.trim_end().to_string())
}

fn create(plug_path: &Path) -> Result<String, String> {
    if symlink_metadata(plug_path).is_ok() {
        return Err(format!("plug already exists: {}", plug_path.display()));
    }

    let plug_name = plug_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("cannot get plug filename")?;

    let plug_pointer_path = PathBuf::from(format!("{SOCKET_PREFIX}{plug_name}"));

    symlink(&plug_pointer_path, plug_path).map_err(|e| format!("cannot create symlink: {e}"))?;

    Ok(format!("Created plug at: {}", plug_path.display()))
}

fn connect(plug_path: &Path, target_path: &Path, relative: bool) -> Result<String, String> {
    validate(plug_path)?;

    let socket_path = socket_path(plug_path)?;

    if symlink_metadata(&socket_path).is_ok() {
        return Err("plug already connected".to_string());
    }

    if !target_path.exists() {
        return Err(format!("target does not exist: {}", target_path.display()));
    }

    let parent = plug_path.parent().ok_or("plug has no parent")?;

    let socket_pointer_path = if relative {
        if target_path.is_absolute() {
            return Err("target must be relative when using --relative".to_string());
        }
        pathdiff::diff_paths(target_path, parent).ok_or("cannot compute relative path")?
    } else if target_path.is_relative() {
        current_dir()
            .map_err(|e| format!("cannot get current directory: {e}"))?
            .join(target_path)
    } else {
        target_path.to_path_buf()
    };

    symlink(&socket_pointer_path, &socket_path)
        .map_err(|e| format!("cannot create socket symlink: {e}"))?;

    Ok(format!(
        "\
        Connected plug\n\
        at {}\n\
        to {}",
        plug_path.display(),
        socket_pointer_path.display()
    ))
}

fn disconnect(plug_path: &Path, delete: bool) -> Result<String, String> {
    validate(plug_path)?;

    let socket_path = socket_path(plug_path)?;

    if symlink_metadata(&socket_path).is_err() {
        return Err(format!("no socket at {}", socket_path.display()));
    }

    if socket_path.is_symlink() {
        remove_file(&socket_path).map_err(|e| format!("cannot remove socket symlink: {e}"))?;
    } else if !delete {
        return Err("plug directly connected to files, use --delete to remove".to_string());
    } else if socket_path.is_file() {
        remove_file(&socket_path).map_err(|e| format!("cannot remove socket file: {e}"))?;
    } else if socket_path.is_dir() {
        remove_dir_all(&socket_path).map_err(|e| format!("cannot remove socket directory: {e}"))?;
    } else {
        return Err("unsupported socket type".to_string());
    }

    Ok(format!("Disconnected plug at {}", plug_path.display()))
}

fn delete(plug_path: &Path) -> Result<String, String> {
    validate(plug_path)?;

    let socket_path = socket_path(plug_path)?;

    if symlink_metadata(&socket_path).is_ok() {
        return Err(format!(
            "plug is connected, disconnect first: {}",
            socket_path.display()
        ));
    }

    remove_file(plug_path).map_err(|e| format!("cannot remove plug symlink: {e}"))?;

    let sample_path = sample_path(plug_path)?;

    if symlink_metadata(&sample_path).is_ok() {
        remove_file(&sample_path).map_err(|e| format!("cannot remove sample file: {e}"))?;
    }

    Ok(format!("Deleted plug at {}", plug_path.display()))
}

fn connect_new_dir(plug_path: &Path) -> Result<String, String> {
    validate(plug_path)?;

    let socket_path = socket_path(plug_path)?;

    if symlink_metadata(&socket_path).is_ok() {
        return Err("plug already connected".to_string());
    }

    create_dir(&socket_path).map_err(|e| format!("cannot create directory: {e}"))?;

    Ok(format!(
        "Connected new directory for plug {}",
        plug_path.display()
    ))
}

fn connect_new_file(plug_path: &Path, from_sample: bool, edit: bool) -> Result<String, String> {
    validate(plug_path)?;

    let socket_path = socket_path(plug_path)?;

    if symlink_metadata(&socket_path).is_ok() {
        return Err("plug already connected".to_string());
    }

    if from_sample {
        let sample_path = sample_path(plug_path)?;

        if !sample_path.exists() {
            return Err(format!(
                "sample file does not exist: {}",
                sample_path.display()
            ));
        }

        copy(&sample_path, &socket_path).map_err(|e| format!("cannot copy sample file: {e}"))?;
    } else {
        File::create(&socket_path).map_err(|e| format!("cannot create socket file: {e}"))?;
    }

    if edit {
        let editor = var("EDITOR").map_err(|_| "EDITOR not set")?;

        Command::new(&editor)
            .arg(&socket_path)
            .status()
            .map_err(|e| format!("editor failed: {e}"))?;
    }

    Ok(format!(
        "Connected new file for plug {}",
        plug_path.display()
    ))
}

fn write_sample_file(plug_path: &Path) -> Result<String, String> {
    validate(plug_path)?;

    let sample_path = sample_path(plug_path)?;

    if sample_path.exists() {
        return Err(format!("sample already exists: {}", sample_path.display()));
    }

    let editor = var("EDITOR").map_err(|_| "EDITOR not set")?;

    File::create(&sample_path).map_err(|e| format!("cannot create sample file: {e}"))?;

    Command::new(&editor)
        .arg(&sample_path)
        .status()
        .map_err(|e| format!("editor failed: {e}"))?;

    Ok(format!(
        "Written sample file for plug {}",
        plug_path.display()
    ))
}
