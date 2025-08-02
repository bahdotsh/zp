# zp P2P Clipboard Sync

A peer-to-peer synchronization system that allows you to sync clipboard history across all your devices, including remote systems accessed via SSH.

## Overview

The zp sync system enables automatic clipboard history synchronization between devices using:
- **HTTP-based sync protocol** for local network devices
- **SSH tunneling support** for remote/virtual systems  
- **Timestamp-based conflict resolution** to merge clipboard histories
- **Manual peer configuration** (no automatic discovery for security)
- **Cross-platform support** (local, remote, virtual machines)

## Quick Start

### 1. Enable Sync
```bash
zp --sync-enable
```

### 2. Add Peers

#### Local network device:
```bash
zp --add-peer laptop:http://192.168.1.100:8080
```

#### Remote device via SSH:
```bash
zp --add-peer server:ssh://user@remote.host:22
```

### 3. Start Sync Daemon
```bash
zp --sync-daemon
```

### 4. Manual Sync (optional)
```bash
zp --sync-now
```

## Peer ID Format

Peer IDs follow the format: `{username}@{hostname}-{random_suffix}`

Examples:
- `goku@macbook-a1b2`
- `user@server-x9z3`
- Custom: `work-laptop`, `home-desktop`

The system auto-generates a unique peer ID on first run, but you can customize it in the config file.

## Configuration

### Config File Location
```
~/.zp/sync_config.json
```

### View Current Configuration
```bash
zp --sync-config
```

### Example Configuration
```json
{
  "enabled": true,
  "peer_id": "goku@macbook-a1b2",
  "listen_port": 8080,
  "peers": {
    "work-laptop": {
      "endpoint": "http://192.168.1.100:8080",
      "enabled": true,
      "ssh_config": null
    },
    "remote-server": {
      "endpoint": "ssh://user@server.com:22",
      "enabled": true,
      "ssh_config": {
        "tunnel_local_port": 8081,
        "remote_port": 8080,
        "ssh_user": "user",
        "ssh_host": "server.com",
        "ssh_port": 22,
        "identity_file": null
      }
    }
  },
  "sync_interval_seconds": 30,
  "auto_sync": true
}
```

## Commands Reference

### Basic Commands
```bash
# Enable/disable sync
zp --sync-enable
zp --sync-disable

# Show configuration
zp --sync-config

# Start sync daemon (runs indefinitely)
zp --sync-daemon

# One-time sync
zp --sync-now
```

### Peer Management
```bash
# Add HTTP peer
zp --add-peer laptop:http://192.168.1.100:8080

# Add SSH peer
zp --add-peer server:ssh://user@remote.host:22

# Remove peer
zp --remove-peer laptop

# Test peer connection
zp --test-peer laptop
```

## Network Setup

### Local Network Devices
For devices on the same network, use HTTP endpoints:
```bash
zp --add-peer laptop:http://192.168.1.100:8080
```

Each device needs to:
1. Run the sync daemon: `zp --sync-daemon`
2. Have firewall rules allowing connections on the sync port (default: 8080)

### Remote/SSH Devices
For devices accessed via SSH (including virtual machines):
```bash
zp --add-peer server:ssh://user@server.com:22
```

The system automatically:
1. Sets up SSH tunnels for secure connections
2. Handles authentication using your SSH config
3. Maps remote sync port to local tunnel port

### SSH Configuration
You can use SSH config file (`~/.ssh/config`) for convenience:
```
Host myserver
    HostName server.com
    User myuser
    Port 22
    IdentityFile ~/.ssh/my_key
```

Then add peer as:
```bash
zp --add-peer server:ssh://myuser@myserver:22
```

## How Sync Works

### Conflict Resolution
- **Timestamp-based merging**: Entries are merged by timestamp order
- **Duplicate prevention**: Identical content is not duplicated
- **Bi-directional sync**: All peers send and receive updates

### Sync Process
1. **Handshake**: Peers establish connection
2. **History request**: Request entries newer than last sync
3. **Merge**: Combine remote entries with local history
4. **Save**: Update local clipboard history file
5. **Response**: Send local updates back to peer

### Data Format
```json
{
  "content": "Hello, world!",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## Troubleshooting

### Connection Issues
```bash
# Test peer connectivity
zp --test-peer laptop

# Check sync configuration
zp --sync-config

# Manual sync with verbose output
zp --sync-now
```

### Common Issues

#### Port Already in Use
If you get port conflicts, edit the config file to change `listen_port`:
```json
{
  "listen_port": 8081
}
```

#### SSH Connection Fails
- Verify SSH key authentication works: `ssh user@host`
- Check SSH config in `~/.ssh/config`
- Ensure remote device is running the sync daemon

#### Firewall Issues
Ensure the sync port (default 8080) is open:
```bash
# Linux (ufw)
sudo ufw allow 8080

# macOS
# Add rule in System Preferences > Security & Privacy > Firewall
```

#### Sync Not Working
1. Check if both devices have sync enabled
2. Verify both are running sync daemons
3. Test network connectivity between devices
4. Check firewall settings

### Debug Steps
1. **Check daemon status**: Look for running sync processes
2. **Verify network**: Test HTTP endpoints directly
3. **SSH troubleshooting**: Test SSH connection manually
4. **Logs**: Check system logs for error messages

## Security Considerations

### Network Security
- HTTP sync is unencrypted (use only on trusted networks)
- SSH tunnels provide encryption for remote connections
- No automatic peer discovery (prevents unauthorized access)

### Best Practices
- Use SSH for connections over untrusted networks
- Regularly review peer list: `zp --sync-config`
- Disable sync when not needed: `zp --sync-disable`
- Use strong SSH key authentication

## Examples

### Home Network Setup
```bash
# Device 1 (laptop)
zp --sync-enable
zp --add-peer desktop:http://192.168.1.50:8080
zp --sync-daemon

# Device 2 (desktop)
zp --sync-enable
zp --add-peer laptop:http://192.168.1.100:8080
zp --sync-daemon
```

### Remote Server Setup
```bash
# Local machine
zp --sync-enable
zp --add-peer server:ssh://user@server.com:22
zp --sync-daemon

# Remote server (via SSH)
ssh user@server.com
zp --sync-enable
zp --add-peer local:ssh://user@local.machine:22
zp --sync-daemon
```

### Mixed Environment
```bash
# Add multiple peers
zp --add-peer laptop:http://192.168.1.100:8080
zp --add-peer server:ssh://user@server.com:22
zp --add-peer vm:ssh://user@vm.host:2222

# Start syncing with all
zp --sync-daemon
```

## Advanced Configuration

### Custom Sync Intervals
Edit `~/.zp/sync_config.json`:
```json
{
  "sync_interval_seconds": 60,
  "auto_sync": true
}
```

### SSH Identity Files
Specify custom SSH keys:
```json
{
  "ssh_config": {
    "identity_file": "/path/to/custom/key"
  }
}
```

### Port Configuration
```json
{
  "listen_port": 8080,
  "peers": {
    "server": {
      "ssh_config": {
        "tunnel_local_port": 8081,
        "remote_port": 8080
      }
    }
  }
}
```

## Integration with zp

The sync system integrates seamlessly with existing zp functionality:
- All clipboard copies are automatically synced
- Clipboard history (`zp -l`) shows merged entries from all devices
- No changes needed to existing workflows

Clipboard sync runs in the background and requires no manual intervention once configured.