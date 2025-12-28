# Plug

> *Making file dependencies explicit and manageable.*

Plug offers a simple way to mark files and directories that aren't part of the repository but need to be provided locally. Instead of silently expecting files to exist, plugs make dependencies explicit through a clean symlink convention.

## Concept

### The Problem

Many projects expect certain files or directories to exist locally, but these requirements are often invisible:
- Configuration files with secrets or local paths
- Data directories that are too large for version control
- Machine-specific settings that vary per developer

When these files are missing, programs fail with cryptic errors. When they're present but not tracked, new contributors don't know what they need to provide.

### The Plug Solution

A **plug** is a symlink with the naming pattern: `foo -> ._.foo`

- `foo` is the file/directory your program expects
- `._.foo` is the "socket" - what you need to provide locally

The `._.` prefix serves multiple purposes:
- It's a dotfile (hidden by default)
- It visually resembles a socket
- It's clear and searchable

### Git Integration

Add `._.` to your `.gitignore`:
```gitignore
._.*
```

This way you commit the plugs (the symlinks) but not the sockets (the actual files). Contributors can see exactly what needs to be provided, while keeping sensitive or local data out of version control.

### Sample Files

Provide `.sample.foo` files as templates:
```
config -> ._.config          # Plug (committed)
.sample.config               # Template (committed)
._.config                    # Socket (not committed, user provides)
```

Users can copy the sample to the socket and customize it for their environment.

## Tool

The `plug` CLI helps manage these symlinks.

### Installation

Build from source:
```bash
cargo build --release
```

Then copy the binary:
```bash
# Global installation
sudo cp target/release/plug /usr/bin

# Local installation (ensure ~/.local/bin is in PATH)
cp target/release/plug ~/.local/bin
```

### Usage

```bash
# List all plugs and their status
plug list

# Create a socket by linking to an existing file/directory
plug link config /path/to/my/config

# Create a socket from a sample file and open in editor
plug sample .sample.config

# Create an empty directory socket
plug mkdir data
```

For detailed help:
```bash
plug --help
```

### Commands

#### `list`
Shows all plugs in the current directory and subdirectories:
```
config -> ._.config [UNPLUGGED]
data -> ._.data [PLUGGED (directory)]
secrets -> ._.secrets [PLUGGED (symlink) -> /home/user/.secrets]
```

Status indicators:
- `UNPLUGGED` - Socket doesn't exist yet
- `PLUGGED (file)` - Socket is a regular file
- `PLUGGED (directory)` - Socket is a directory
- `PLUGGED (symlink) -> <path>` - Socket is itself a symlink

#### `link`
Creates a socket by symlinking to an existing file or directory:
```bash
plug link config ~/.myapp/config
# Creates: ._.config -> /home/user/.myapp/config
```

#### `sample`
Copies a sample file to create a socket, then opens it in your editor:
```bash
plug sample .sample.config
# Creates ._.config from .sample.config
# Opens ._.config in $EDITOR (or $VISUAL, or vi)
```

#### `mkdir`
Creates an empty directory socket:
```bash
plug mkdir data
# Creates: ._.data/ (empty directory)
```

## Example Workflow

Setting up a new project with plugs:
```bash
# Clone the repository
git clone https://github.com/user/project.git
cd project

# See what needs to be provided
plug list
# config -> ._.config [UNPLUGGED]
# data -> ._.data [UNPLUGGED]

# Create config from sample
plug sample .sample.config

# Create empty data directory
plug mkdir data

# Verify everything is plugged
plug list
# config -> ._.config [PLUGGED (file)]
# data -> ._.data [PLUGGED (directory)]

# Run the application
./app
```
