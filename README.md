# Plugs

<img align="right" src="logo.svg" width="200" align="right" alt="Plugs Logo">

> *Making file dependencies explicit and manageable through a symlink-based framework.*

Plugs offer a structured way to declare and manage file dependencies in your projects using symbolic links.
By treating dependencies as "plugs" that can be connected to different "sockets", this framework makes external file dependencies visible, auditable, and easy to reconfigure.

## Concept

Plugs make files dependencies explicit and portable:
```
config.json -> ._.config.json -> /etc/myapp/config.json  # plug committed, socket local
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

Plugs make dependencies explicit through a three-layer system:

1. **Plug**: A symlink named after the dependency (e.g., `config.json`) that points to a socket (`._.config.json`)
2. **Socket**: Either the actual file/directory, or a symlink pointing to the external dependency
3. **Sample** (optional): A sample file (`.~.config.json`) showing what the dependency should look like

**How It Works**:

1. **Declaration**: Create a plug symlink `config.json -> ._.config.json`
2. **Connection**: Create the socket `._.config.json` as either:
   - A symlink to an external file/directory
   - A new local file/directory
3. **Usage**: Your code uses `config.json` normally; the plug-socket layer is transparent

Here's how a generic project looks with plugs:

```
project/
│
├─── config.json -> ._.config.json
├─── ._.config.json -> /etc/myapp/config.json
├─── .~.config.json (sample configuration)
│
├─── database.db -> ._.database.db
├─── ._.database.db -> /var/lib/myapp/db.sqlite
│
├─── .env -> ._..env
├─── ._..env
└─── .~..env (sample environment variables)
```

See `example` for a working example of this structure.

To create a plug manually:

```sh
# Create the plug
ln -s ._.config.json config.json

# Create the socket pointing to the external file
ln -s /etc/myapp/config.json ._.config.json

# Optionally create a sample file
cp /etc/myapp/config.json .~.config.json
```

**Benefits**:

- **Explicit dependencies**: Plugs (committed to git) declare "this file is needed here"
- **Out-of-the-box discovery**: When cloning the repository, all plugs point to non-existent sockets, making them immediately visible with a simple command
- **Templating**: Sample files show exactly what each dependency should contain
- **Flexible connection**: Connect to system files, user files, or create local copies; the plug stays the same
- **Environment independence**: Different developers/environments can connect plugs to different locations without modifying the repository
- **Status visibility**: Easy to audit which dependencies are connected, disconnected, or broken

**Naming Convention**:
- `._.` prefix: "socket" - the connection point (gitignored, local to each environment).
- `.~.` prefix: "sample" - example/template (committed, shows what's expected).

Both use dot-prefixes, making them hidden files that don't clutter your regular directory view while keeping them close to their plugs.

> *The `._.` prefix visually resembles a socket!*

**Git Configuration**:

Add to your `.gitignore`:

```gitignore
# Plug sockets
._.* 
```

This ensures that:
- **Plugs** (`config.json -> ._.config.json`) are committed: declarations of what's needed
- **Sockets** (`._.config.json`) are gitignored: local connections specific to each environment

## Shell Command

The repository also includes a ready-to-use command `plug` that simplifies plugs management.

### Installation

**Download and extract**:

```sh
wget https://github.com/niqodea/plugs/releases/download/v0.1.0/plugs-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
tar -xzf plugs-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
```

Then `cp` the `plug` binary to the `bin` directory.

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
plug create src/config.json
plug connect src/config.json --to /etc/myapp/config.json
```

To check the status of all plugs in your project:

```sh
plug status
```
