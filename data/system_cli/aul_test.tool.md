# aul_test

Tizen AUL (Application Utility Library) test tool. Provides command-line access to app lifecycle management APIs for querying, launching, pausing, resuming, and terminating applications.

**Binary**: `/usr/bin/aul_test`
**Category**: system_cli

## Usage

```
aul_test <subcommand> [arguments...]
```

## Key Subcommands

### Query Running Apps
| Subcommand | Arguments | Description |
|------------|-----------|-------------|
| `getallpkg` | *(none)* | List all currently running apps with app IDs and PIDs |
| `getallappstatus` | *(none)* | List all running apps with their status (fg/bg/ready/dying) |
| `is_run` | `<appid>` | Check if a specific app is currently running |
| `get_pid` | `<appid>` | Get the PID of a running app |
| `get_status` | `<appid>` | Get the status of a specific app |
| `get_status_pid` | `<pid>` | Get app status by PID |

### App Info Lookup
| Subcommand | Arguments | Description |
|------------|-----------|-------------|
| `getpkg` | `<appid>` | Get app info from database by app ID |
| `get_app_bypid` | `<pid>` | Get the app ID from a PID |
| `get_pkg_bypid` | `<pid>` | Get the package ID from a PID |
| `get_app_lifecycle` | `<appid>` | Get app lifecycle state |
| `get_cpu_usage` | `<pid>` | Get CPU usage for a process |
| `get_proc_name` | `<pid>` | Get process name by PID |
| `pkg_install_status` | `<pkgname>` | Get package install status |

### App Launch & Control
| Subcommand | Arguments | Description |
|------------|-----------|-------------|
| `launch` | `<app_id> [key1 val1 ...]` | Launch an app with optional key-value extras |
| `open` | `<appid>` | Open (bring to foreground) an app |
| `pause` | `<app_id>` | Pause an app |
| `resume` | `<appid>` | Resume a paused app |
| `term_pid` | `<pid>` | Terminate an app by PID |
| `terminate_app_async` | `<appid>` | Terminate an app asynchronously by app ID |

### MIME & Default Apps
| Subcommand | Arguments | Description |
|------------|-----------|-------------|
| `get_default_app` | `<operation> <uri> <mime>` | Get the default app for a given operation/URI/MIME |
| `get_defapp_mime` | `<mime_type>` | Get default app for a MIME type |
| `get_mime_file` | `<filename>` | Get MIME type from a file |
| `get_mime_content` | `<content>` | Get MIME type from content |

### Process Groups
| Subcommand | Arguments | Description |
|------------|-----------|-------------|
| `foreach_proc_group` | *(none)* | List all process groups |
| `get_proc_group` | `<pid>` | Get process group for a PID |

## Examples

```bash
# List all running apps
aul_test getallpkg

# Check if Settings app is running
aul_test is_run org.tizen.setting

# Get PID of an app
aul_test get_pid org.tizen.setting

# Launch an app
aul_test launch org.tizen.setting

# Terminate app by PID
aul_test term_pid 1234

# Get detailed status of all running apps
aul_test getallappstatus
```

## Output Format

Most query commands output plain text with tab-separated values. Example `getallpkg` output:
```
======================
pid     appid
------  ------
1234    org.tizen.setting
5678    org.tizen.homescreen
======================
```
