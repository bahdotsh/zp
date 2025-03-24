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
