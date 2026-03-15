# winfo

Tizen window compositor (Enlightenment) information and control tool. Query window hierarchy, properties, screen info, and perform compositor-level operations.

**Binary**: `/usr/bin/winfo`
**Category**: system_cli

## Usage

```
winfo -<command> [arguments...]
```

## Key Commands

### Window Information (Read-Only)
| Command | Arguments | Description |
|---------|-----------|-------------|
| `-topwins` | *(none)* | List all top-level windows |
| `-topvwins` | *(none)* | List top visible windows |
| `-wininfo` | `[options]` | Detailed window information (geometry, parent/child, etc.) |
| `-compobjs` | `[simple]` | List compositor objects |
| `-view` | *(none)* | Show view hierarchy |
| `-view_tree_info` | *(none)* | Print view tree information |
| `-subsurface` | *(none)* | Show subsurface information |
| `-focus_history` | *(none)* | Show focus history |

### Window Properties
| Command | Arguments | Description |
|---------|-----------|-------------|
| `-prop` | `<win_id> [property] [value]` | Get/set window properties |
| `-prop_set` | `<win_id> <prop> <value>` | Set window property (pin, modal, skip_zoom) |

### System Info (Read-Only)
| Command | Arguments | Description |
|---------|-----------|-------------|
| `-screen_info` | *(none)* | Display screen resolution and configuration |
| `-fps` | `[-win_id <id>]` | Show FPS (frames per second) |
| `-connected_clients` | *(none)* | List connected Wayland clients |
| `-input_devices` | *(none)* | List input devices |
| `-input_region` | *(none)* | Show input regions |
| `-keygrab_status` | *(none)* | Show key grab status |
| `-keymap` | *(none)* | Show current keymap |
| `-module_info` | *(none)* | Show loaded compositor modules |
| `-process_info` | *(none)* | Show compositor process information |
| `-version` | *(none)* | Show compositor version |
| `-reslist` | `[-tree\|-p <pid>]` | List Wayland resources |
| `-hwc_wins` | *(none)* | Show HWC window status |
| `-container_info` | *(none)* | Show container info |

### Display Control
| Command | Arguments | Description |
|---------|-----------|-------------|
| `-rotation` | `set <zone-no> <angle>` | Set screen rotation (0/90/180/270) |
| `-screen_rotation` | `<0\|90\|180\|270>` | Rotate screen |
| `-desk` | `geometry <x> <y> <w> <h>` | Change desktop geometry |
| `-desk` | `zoom <zx> <zy> <cx> <cy>` | Scale desktop screen |
| `-effect` | `<1\|0>` | Enable/disable window effects |
| `-hwc` | `<1\|0\|2>` | HWC control (on/off/info) |
| `-magnifier` | *(none)* | Magnifier control |

### Screen Saver
| Command | Arguments | Description |
|---------|-----------|-------------|
| `-scrsaver` | `info` | Get screen saver info |
| `-scrsaver` | `enable` | Enable screen saver |
| `-scrsaver` | `disable` | Disable screen saver |
| `-scrsaver` | `timeout <sec>` | Set screen saver timeout |

### Visual Effects
| Command | Arguments | Description |
|---------|-----------|-------------|
| `-blur_option` | `<type> <on\|off> [opts]` | Control blur/dim/stroke/shadow effects |
| `-filter` | `<win_id> <type> <on\|off>` | Apply visual filters (blur, grayscale, inverse_color) |
| `-bgcolor_set` | `[a,r,g,b]` | Set background color |

### Window Operations
| Command | Arguments | Description |
|---------|-----------|-------------|
| `-basic_op_gen` | `<win_id> <operation>` | Window operations: lower, activate, iconify, uniconify |
| `-quickpanel` | `<0\|1\|2\|3> [win_id]` | Quickpanel control (hide/show/lock/unlock) |
| `-aux_hint` | `<win> <id> <hint> <value>` | Set auxiliary hint on window |
| `-aux_msg` | `<win> <key> <value> [opts]` | Send auxiliary message to window |

### Debug & Dumping
| Command | Arguments | Description |
|---------|-----------|-------------|
| `-protocol_trace` | `[console\|file\|disable]` | Enable/disable Wayland protocol tracing |
| `-dump_screen` | `-p /tmp/ -n xxx.png` | Dump screen to PNG file |
| `-dump_buffers` | `<1\|0> [options]` | Start/stop buffer dumping |
| `-dump_memory` | *(none)* | Dump memory info to /tmp |

### `-wininfo` Options
```
-children    : print parent and child identifiers
-tree        : print children identifiers recursively
-stats       : print window geometry [DEFAULT]
-id <win_id> : use the window with specified id
-name <name> : use the window with specified name
-pid <pid>   : use the window with specified pid
-int         : print window id in decimal
-size        : print size hints
-wm          : print window manager hints
-shape       : print shape rectangles
-all         : -tree, -stats, -wm, -size, -shape
```

## Examples

```bash
# List all top-level windows
winfo -topwins

# List visible windows
winfo -topvwins

# Get detailed info for all windows
winfo -wininfo

# Get info for a specific window by ID
winfo -wininfo -id 0xb88ffaa0 -all

# Get info for windows belonging to a PID
winfo -wininfo -pid 1234 -stats

# Show screen info
winfo -screen_info

# Show connected clients
winfo -connected_clients

# Show FPS
winfo -fps

# Show input devices
winfo -input_devices

# Rotate screen 90 degrees
winfo -screen_rotation 90

# Get screen saver status
winfo -scrsaver info

# Dump screen to file
winfo -dump_screen -p /tmp/ -n screenshot.png

# Get window properties
winfo -prop -name "" 
```
