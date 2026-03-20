# tizen-file-manager-cli
**Description**: File system operations — read, write, append, copy, move, remove, list, stat, mkdir.

## Usage
```
tizen-file-manager-cli <subcommand> [options]
```

## Subcommands

| Subcommand | Description | Required Args |
|------------|-------------|---------------|
| `read` | Read file contents | `--path` |
| `write` | Write/overwrite file | `--path --content` |
| `append` | Append to file | `--path --content` |
| `remove` | Remove file | `--path` |
| `mkdir` | Create directory | `--path` |
| `list` | List directory entries | `--path` |
| `stat` | Get file/dir metadata | `--path` |
| `copy` | Copy file | `--src --dst` |
| `move` | Move/rename file | `--src --dst` |

## Examples
```bash
# Read a file
tizen-file-manager-cli read --path /tmp/test.txt

# Write a file
tizen-file-manager-cli write --path /tmp/out.txt --content "hello world"

# List directory
tizen-file-manager-cli list --path /opt/usr/share

# Copy file
tizen-file-manager-cli copy --src /tmp/a.txt --dst /tmp/b.txt
```

## Output
All output is JSON.
