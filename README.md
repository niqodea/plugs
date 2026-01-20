# Plugs

<img src="logo.svg" width="100" align="right" alt="Plugs Logo">

> *Making file dependencies explicit and manageable through a symlink-based framework.*

Plugs offer a structured way to declare and manage file dependencies using symbolic links.

## Concept

Plugs make file dependencies explicit and portable:

```sh
# plug (committed)
ln -s ._.config.json config.json

# socket (not committed)
ln -s /etc/myapp/config.json ._.config.json 

# works transparently
cat config.json
```

### The Problem

Many projects rely on external files that aren't part of the repository itself: configuration files in `/etc`, databases in `/var`, user-specific settings in `~/.config`, or shared resources in other directories.
These dependencies are typically handled in one of several ways:

- **Environment variables**: Pass paths like `CONFIG_PATH=/etc/myapp/config.json` at runtime.
  Dependencies only discovered when the program runs and fails.
- **Expected files**: Programs simply assume files exist (like `.env` in the project root).
  Users must somehow know to create them.
- **Hard-coded paths**: Paths baked into the code.
  Inflexible and environment-specific.
- **Documentation**: README instructions listing what to create/configure.
  Quickly becomes outdated and requires careful manual setup.
- **Direct symlinks**: Quick to create but invisible in the repository.
  No way to know what's supposed to be linked to what.
- **Configuration management**: Ansible, Chef, etc.
  Often overkill for simple file dependencies.

The core issue is that **file dependencies are implicit**.
When someone sets up your project, they have no systematic way to discover what external files are needed, where they should come from, or what they should contain.
The dependency only reveals itself at runtime through an error message or silent misconfiguration.

### Plugs Approach

Plugs make dependencies explicit through a two-layer system:

1. **Plug**: A symlink named after the dependency (e.g., `config.json`) that points to a socket (`._.config.json`)
2. **Socket**: The actual file/directory, or a symlink pointing to the external dependency

Plugs are committed, sockets are not. Your code uses the plug path normally; the indirection is transparent.

Optionally, a sample file (`.~.config.json`) shows expected structure.
Copy it to the socket to bootstrap setup.

The `._.` and `.~.` prefixes keep these files hidden but colocated with their plugs. *(And `._.` visually resembles a socket!)*

**Benefits**:

- **Explicit dependencies**: Plugs declare "this file is needed here"; fresh clones have dangling symlinks that reveal missing dependencies before runtime
- **Templating**: Sample files show exactly what each dependency should contain
- **Flexible connection**: Connect to system files, user files, or create local copies; the plug stays the same
- **Status visibility**: Easy to audit which dependencies are connected, disconnected, or broken
- **Self-contained context**: Your editor's file tree becomes a complete map of everything the project touches

**Git Configuration**:

Add to your `.gitignore`:

```gitignore
# Plug sockets
._.* 
```

This ensures that:
- **Plugs** (`config.json -> ._.config.json`) are committed: declarations of what's needed
- **Sockets** (`._.config.json`) are gitignored: local connections specific to each environment

**Example Project**

Here's how a generic project looks with plugs:

```
myapp/
│
├─── .gitignore (ignores sockets)
│
├─── config.json -> ._.config.json
├─── ._.config.json -> /etc/myapp/config.json
├─── .~.config.json (sample configuration)
│
├─── database.db -> ._.database.db
├─── ._.database.db -> /var/lib/myapp/db.sqlite
│
├─── .env -> ._..env
├─── ._..env (local file)
└─── .~..env (sample environment variables)
```

See the `example` directory for a working example.

## The `plug` Command

The repository includes a `plug` command that simplifies plug management.

### Installation

**Download and extract**:

```sh
wget https://github.com/niqodea/plugs/releases/download/v0.3.0/plugs-v0.3.0-x86_64-unknown-linux-gnu.tar.gz
tar -xzf plugs-v0.3.0-x86_64-unknown-linux-gnu.tar.gz
```

Then copy the `plug` binary to a directory in your `PATH`.

- **Global Installation**:
   ```sh
   sudo cp plug /usr/bin
   ```

- **Local Installation**:
   First, ensure `~/.local/bin` is in your `PATH`. Then:
   ```sh
   cp plug ~/.local/bin
   ```

### Usage

Refer to the command's help message:

```sh
plug --help
```

For example, to create a plug and connect it to an existing file:

```sh
plug create config.json
plug connect config.json --to /etc/myapp/config.json
```

To check the status of all plugs in your project:

```sh
plug status
```
