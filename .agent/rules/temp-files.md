# Temporary Files Rule

When temporary files are needed in the project, the following rules **must** be followed.

## Rules

1. **Location**: Use the `.tmp/` directory at the workspace root.
   - Create the directory if it does not exist: `mkdir -p .tmp`
   - Use the project-local `.tmp/`, not system temp directories like `/tmp/`.

2. **Cleanup**: Temporary files **must be deleted** once they are no longer needed.
   - Delete individual files or clean up with `rm -rf .tmp/*`.

3. **Git exclusion**: The `.tmp/` directory is registered in `.gitignore` and will not be committed.
