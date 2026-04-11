# Devel Progress Telegram Streaming And Dashboard Gate Design

## Cycle

- Classification: host-default
- Build path: `./deploy_host.sh`

## Runtime Surface

- `src/tizenclaw/src/core/devel_mode.rs`
- `src/tizenclaw/src/main.rs`
- `deploy_host.sh`
- `data/config/channel_config.json`
- `tests/system/devel_mode_prompt_flow.json`

## Design Summary

- Extend devel runtime paths with `~/.tizenclaw/devel/progress`.
- Watch both `progress/` and `result/` with inotify inside devel mode.
- Start streaming only for files named `<prompt>_progress.log`.
- Forward appended log chunks to Telegram while the matching
  `<prompt>_RESULT.md` does not exist in `result/`.
- Stop streaming that prompt immediately once the matching result file is
  created, while preserving the existing result-file outbound notification.
- Keep dashboard registered but disable auto-start by default so it only
  runs when explicitly started through `tizenclaw-cli dashboard start`.

## Ownership And Boundaries

- `devel_mode.rs` owns devel filesystem watching and Telegram outbound.
- Channel registry keeps `web_dashboard` available for on-demand startup.
- `deploy_host.sh` must not silently re-enable dashboard auto-start.

## Persistence And Observability

- Devel status JSON should expose `progress_dir` and watcher activity.
- System scenario should assert the new progress path is exposed through IPC.
