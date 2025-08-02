# zp

`zp` is a command-line tool designed to copy text or data from files or standard output buffers directly to the clipboard. This can be particularly useful for developers and power users who need quick access to snippets of code, configurations, or any other textual information.

## Usage

### Copying from a File
To copy the entire contents of a file:
```bash
zp my_file.txt
```

To get the first `n` words from a file:
```bash
zp -s n my_file.txt
```

To get lines between a range, i.e., lines `n` to `m`:
```bash
zp -s n -e m my_file.txt
```

### Copying from Standard Output
To copy the entire output of a command:
```bash
cat sample_file.txt | zp
```

For ranges and specific words, you can use similar flags as with files.

## Logs and History

Every copied content is saved to a history file located in your home directory (`~/.zp/clipboard_history.json`). You can view the copy history using:
```bash
zp --logs
```
This provides an interactive interface showing the last copied items, along with timestamps. The log viewer supports navigation, copying, and exiting.

## Daemon Mode

The `zp` tool also includes a clipboard monitoring daemon to automatically save any changes made to the clipboard. This can be especially useful if you want to keep a history of every change made to your clipboard without manually triggering the copy command each time.

### Starting the Daemon
To start the clipboard monitoring daemon:
```bash
zp --daemon
```
This will fork the tool into the background and continue running, automatically saving any clipboard changes.

### Stopping the Daemon
To stop the clipboard monitoring daemon:
```bash
zp --stop-daemon
```

### Checking the Daemon Status
You can check if the daemon is currently running with:
```bash
zp --daemon-status
```
This will inform you whether the daemon is active and provide its process ID.

## Sync Mode

`zp` includes a powerful peer-to-peer synchronization system that allows you to sync clipboard history across all your devices, including remote systems accessed via SSH.

### Quick Start with Sync

#### 1. Enable Sync
```bash
zp --sync-enable
```

#### 2. Add Peers
For local network devices:
```bash
zp --add-peer laptop:http://192.168.1.100:8080
```

For remote devices via SSH:
```bash
zp --add-peer server:ssh://user@remote.host:22
```

#### 3. Start Sync Daemon
```bash
zp --sync-daemon
```
This starts the sync daemon in the background, enabling automatic clipboard synchronization.

#### 4. Sync Management
```bash
# Check sync daemon status
zp --sync-daemon-status

# Stop sync daemon
zp --stop-sync-daemon

# Manual sync
zp --sync-now

# View sync configuration
zp --sync-config
```

### Sync Features

- **Cross-platform support**: Works on local networks and remote systems
- **SSH tunneling**: Secure sync with remote devices via SSH
- **Background operation**: Sync daemon runs in the background
- **Conflict resolution**: Timestamp-based merging prevents data loss
- **Manual control**: Start, stop, and check status of sync processes

For detailed sync setup and configuration, see [SYNC_USAGE.md](SYNC_USAGE.md).

## Installation

The recommended way to install `zp` is using Rust's package manager, Cargo. Here are several methods:

### Using Cargo Install (Recommended)
```bash
cargo install zp
```

### From Source
For those who prefer building from source, you can clone the repository and build it manually:
```bash
git clone https://github.com/bahdotsh/zp.git
cd zp
cargo install --path .
```
This method allows you to customize or modify the code before installation.

## License

`zp` is licensed under the MIT license. See the `LICENSE` file for more details.
