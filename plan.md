# ZP Clipboard History Sync Implementation Plan

## Overview
This document outlines the plan for implementing a feature that allows ZP to synchronize clipboard history across multiple devices using Syncthing or similar peer-to-peer file synchronization technologies. This feature will enable users to maintain a consistent clipboard history across all their devices.

## Current Architecture Analysis

ZP currently has:
- A clipboard daemon that runs in the background and monitors clipboard changes
- Local storage of clipboard history in `~/.zp/clipboard_history.json`
- A structured format for clipboard entries with content and timestamps
- Functions for saving, loading, and displaying clipboard history

## Implementation Approach

### 1. Syncing Architecture

We'll implement the sync feature using a layered approach:

1. **Local Storage Layer** (existing) - Manages local clipboard history
2. **Sync Layer** (new) - Handles file synchronization between devices
3. **Conflict Resolution Layer** (new) - Resolves conflicts between devices with different histories
4. **Configuration Layer** (new) - Manages user preferences for syncing

### 2. Technology Selection

#### Syncthing Integration Options

1. **External Syncthing Dependency**
   - Require users to install and configure Syncthing separately
   - ZP would simply use the synced folders that Syncthing maintains
   - Pros: Simpler implementation, leverages Syncthing's full capabilities
   - Cons: Additional user setup, dependency on external software

2. **Embedded Syncthing Library**
   - Include Syncthing functionality directly in ZP
   - Use Syncthing's Go implementation through FFI or a Rust port
   - Pros: Seamless user experience, no external dependencies
   - Cons: More complex implementation, maintenance burden

3. **Custom Sync Protocol**
   - Implement our own simplified sync protocol specific to clipboard data
   - Pros: Tailored to our needs, potentially lighter weight
   - Cons: Much more development work, security and reliability concerns

**Recommendation:** Option 1 for initial implementation, with the possibility to move to Option 2 in the future.

### 3. Data Structure Changes

```rust
// Enhanced ClipboardHistoryEntry with device identifier
pub struct ClipboardHistoryEntry {
    pub content: String,
    pub timestamp: String,
    pub device_id: String,  // New field to identify source device
    pub sync_status: SyncStatus, // New field to track sync status
    pub entry_id: String,   // Unique ID for conflict resolution
}

pub enum SyncStatus {
    LocalOnly,
    Synced,
    Conflicted,
}

// Configuration for sync functionality
pub struct SyncConfig {
    pub enabled: bool,
    pub sync_dir: PathBuf,
    pub device_name: String,
    pub auto_merge: bool,
    pub conflict_resolution: ConflictResolutionStrategy,
}

pub enum ConflictResolutionStrategy {
    KeepNewest,
    KeepBoth,
    PreferLocalDevice,
    PreferSpecificDevice(String),
}
```

### 4. Implementation Steps

#### Phase 1: Preparation and Configuration

1. Add sync configuration options to ZP
   - Enable/disable syncing
   - Specify sync directory
   - Set device name
   - Configure conflict resolution strategy

2. Modify the clipboard history data structure
   - Add device identifier to each entry
   - Add unique entry IDs for conflict resolution
   - Add sync status field

3. Create a config file for sync settings
   - Store in `~/.zp/sync_config.json`

#### Phase 2: Basic Sync Functionality

4. Implement file watching for the sync directory
   - Monitor changes to the synced clipboard history file
   - Use notify crate or similar for file system events

5. Implement sync file format and conversion
   - Define a serialization format for the sync file
   - Create functions to convert between local and sync formats

6. Modify save_clipboard_history to update the sync file
   - When a new clipboard entry is added, update both local and sync files

7. Implement import from sync file
   - Add functionality to read the sync file and update local history

#### Phase 3: Conflict Resolution and Robustness

8. Implement conflict detection
   - Identify when the same content has been copied on multiple devices
   - Detect when histories have diverged

9. Add conflict resolution strategies
   - Timestamp-based (newest wins)
   - Device priority
   - Keep both entries

10. Add error handling and recovery
    - Handle sync file corruption
    - Implement history repair mechanisms

11. Add sync status indicators in the UI
    - Show sync status of each clipboard entry
    - Display any conflicts that need resolution

#### Phase 4: User Experience and Testing

12. Create documentation for setup and usage
    - Instructions for configuring Syncthing
    - Recommendations for sync settings

13. Improve sync performance
    - Optimize sync file format for quick updates
    - Implement incremental sync for large histories

14. Test sync functionality across different platforms
    - Ensure compatibility between macOS, Linux, Windows

## Syncthing Setup Guide (For Documentation)

1. Install Syncthing on all devices
2. Create a dedicated folder for ZP sync
3. Share the folder across devices
4. Configure ZP to use the synced folder

## Technical Challenges and Solutions

### Challenge 1: Race Conditions
When multiple devices update their clipboard simultaneously, race conditions could corrupt the sync file.

**Solution:** Implement file locking or a transactional update mechanism.

### Challenge 2: Large Clipboard History
Syncthing might struggle with very frequent updates to files.

**Solution:** Batch updates and implement a sync cooldown period.

### Challenge 3: Different Platforms
Platform-specific clipboard behavior could cause inconsistencies.

**Solution:** Abstract platform differences in the sync layer and normalize data formats.

### Challenge 4: Network Constraints
Intermittent connectivity could lead to sync failures.

**Solution:** Implement robust retry mechanisms and conflict resolution.

## Implementation Checklist

### Phase 1: Preparation
- [ ] Design and implement enhanced ClipboardHistoryEntry struct
- [ ] Create SyncConfig struct and configuration file
- [ ] Add command-line options for sync configuration
- [ ] Update JSON serialization/deserialization for new structures
- [ ] Create device identification mechanism

### Phase 2: Basic Functionality
- [ ] Implement sync directory file watching
- [ ] Create sync file format and conversion utilities
- [ ] Modify clipboard history saving to update sync file
- [ ] Implement import mechanism from sync file
- [ ] Add basic merge functionality for histories from different devices

### Phase 3: Conflict Resolution
- [ ] Implement conflict detection algorithm
- [ ] Add configurable conflict resolution strategies
- [ ] Create sync status tracking for entries
- [ ] Implement UI indicators for sync status
- [ ] Add manual conflict resolution interface

### Phase 4: Robustness and User Experience
- [ ] Implement error handling for sync failures
- [ ] Add recovery mechanisms for corrupted sync files
- [ ] Create comprehensive logging for sync events
- [ ] Write user documentation for setup and usage
- [ ] Perform cross-platform testing

### Phase 5: Optimization and Refinement
- [ ] Optimize sync performance for large histories
- [ ] Implement bandwidth and resource usage controls
- [ ] Add sync statistics and monitoring
- [ ] Perform security review of sync implementation
- [ ] Add automated tests for sync functionality

## Future Enhancements
- End-to-end encryption for clipboard content
- Selective sync for sensitive clipboard entries
- Web interface for accessing clipboard history remotely
- Advanced filtering and search capabilities across synchronized history
- Direct device-to-device sync without relying on a sync directory

## Resource Requirements
- Development time: Approximately 4-6 weeks
- Testing resources: Multiple devices across different platforms
- Additional dependencies: file system notification library, possibly crypto libraries
- Documentation: User guide and setup instructions

## Conclusion
This implementation plan provides a comprehensive approach to adding clipboard history synchronization to ZP. By leveraging existing technologies like Syncthing and implementing robust conflict resolution, we can provide users with a seamless clipboard history experience across all their devices. 