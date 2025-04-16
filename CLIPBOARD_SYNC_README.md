# ZP Clipboard Sync Feature

This document provides instructions for using the clipboard history synchronization feature in ZP.

## Overview

ZP now includes the ability to synchronize your clipboard history across multiple devices. This feature uses a file-based approach that can work independently or alongside external sync tools like Dropbox, Google Drive, or Syncthing.

## Setup

### 1. Enable Sync

To enable clipboard history syncing:

```
zp --sync-enable
```

### 2. Set Device Name (Optional)

Set a unique name for this device:

```
zp --sync-device-name "MyLaptop"
```

### 3. Set Sync Directory (Optional)

By default, ZP will create a sync directory at `~/.zp/sync`. You can change this:

```
zp --sync-dir "/path/to/sync/directory"
```

### 4. Restart the Daemon

After changing sync settings, restart the ZP daemon:

```
zp --stop-daemon && zp --daemon
```

### 5. Check Sync Status

To view the current sync status:

```
zp --sync-status
```

## Using Across Multiple Devices

1. Install ZP on each device.
2. Enable sync on each device with a unique device name.
3. Set the sync directory to the same location across devices (if using an external sync tool, make sure it's a folder that's synchronized).
4. Start the ZP daemon on each device.
5. Copy content on any device, and it will automatically sync to all other devices.

## How Sync Works

1. When you copy text to the clipboard, ZP saves it to your local history and also writes it to the sync directory.
2. ZP monitors the sync directory for changes from other devices.
3. When changes are detected, ZP merges them with your local history according to your conflict resolution settings.
4. ZP uses heartbeat files to detect other active devices on the network.

## Conflict Resolution

When the same clipboard entry is modified on multiple devices, conflicts are resolved according to your configuration settings. By default, the newest entry will be kept.

## Troubleshooting

### Sync Not Working

1. Check if sync is enabled: `zp --sync-status`
2. Ensure the daemon is running: `zp --daemon-status`
3. Verify that the sync directory exists and is accessible
4. If using an external sync tool, verify that it's correctly synchronizing the directory
5. Restart the daemon: `zp --stop-daemon && zp --daemon`

### Reset Sync

To disable sync and start fresh:

```
zp --sync-disable
zp --stop-daemon
```

Then remove the sync directory (`~/.zp/sync` by default).

## Advanced Configuration

For advanced configuration options not available through command-line arguments, edit the sync configuration file directly:

```
~/.zp/sync_config.json
```

## Using with External Sync Tools

ZP's clipboard sync works best when paired with an external file synchronization tool:

1. **Dropbox/Google Drive**: Set your sync directory to a location within your Dropbox or Google Drive folder.
2. **Syncthing**: Set up Syncthing to synchronize your ZP sync directory across devices.
3. **GitHub/Git**: For advanced users, your sync directory could be a git repository that you push/pull manually.

## Security Considerations

- Clipboard history may contain sensitive data like passwords or personal information
- The security of your synchronized data depends on the security of your chosen sync method
- Consider encrypting sensitive clipboard data if sharing across public networks 