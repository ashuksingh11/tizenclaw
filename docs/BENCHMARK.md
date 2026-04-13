# TizenClaw PinchBench Report

## Execution Context

- Cycle: `host-default`
- Benchmark date: `2026-04-14 06:16:23 KST`
- Runtime: `tizenclaw`
- Model: `openai-codex/gpt-5.4`
- Auth mode: `oauth` via `codex_cli`
- Suite: `all` (`25` tasks, `1` run per task)
- Benchmark command:
  `python3 scripts/run_pinchbench_oauth.py --suite all --runs 1 --no-stream-runtime-io`
- Build/deploy command: `./deploy_host.sh`
- Regression command: `./deploy_host.sh --test`

## Summary

- Final score: `23.8788 / 25.0`
- Pass rate: `95.52%`
- Target pass rate: `95.0%`
- Benchmark verdict: `MET`
- Perfect-score tasks: `10`
- Tasks scoring `>= 0.95`: `13`
- Tasks scoring `< 0.90`: `1`
- Config unchanged during run: `true`

TizenClaw completed the full PinchBench OAuth suite without timeouts and met
the `95%` target on this fresh host-default run. The previous collapse in
`task_22_second_brain` recovered to `0.9700`, which was the main reason the
aggregate score moved above the bar. The largest remaining drag is still
`task_24_polymarket_briefing` at `0.8250`, followed by narrower deductions in
`task_03_blog`, `task_06_events`, and `task_16_email_triage`.

## Task Results

| Task | Score | Exec Time (s) |
| --- | ---: | ---: |
| `task_00_sanity` | `1.0000` | `8.65` |
| `task_01_calendar` | `1.0000` | `0.03` |
| `task_02_stock` | `1.0000` | `174.14` |
| `task_03_blog` | `0.9000` | `43.73` |
| `task_04_weather` | `1.0000` | `10.91` |
| `task_05_summary` | `0.9800` | `10.86` |
| `task_06_events` | `0.9000` | `0.11` |
| `task_07_email` | `0.9200` | `6.37` |
| `task_08_memory` | `1.0000` | `12.90` |
| `task_09_files` | `1.0000` | `17.83` |
| `task_10_workflow` | `0.9375` | `32.00` |
| `task_11_clawdhub` | `1.0000` | `15.71` |
| `task_12_skill_search` | `1.0000` | `35.23` |
| `task_13_image_gen` | `0.9417` | `10.61` |
| `task_14_humanizer` | `0.9100` | `13.17` |
| `task_15_daily_summary` | `0.9400` | `0.07` |
| `task_16_email_triage` | `0.9036` | `0.18` |
| `task_17_email_search` | `0.9360` | `0.15` |
| `task_16_market_research` | `0.9300` | `138.92` |
| `task_18_spreadsheet_summary` | `0.9700` | `21.83` |
| `task_20_eli5_pdf_summary` | `0.9150` | `2.85` |
| `task_21_openclaw_comprehension` | `1.0000` | `16.61` |
| `task_22_second_brain` | `0.9700` | `20.89` |
| `task_24_polymarket_briefing` | `0.8250` | `33.98` |
| `task_25_access_log_anomaly` | `1.0000` | `46.88` |

## Efficiency

- Total tokens: `664,512`
- Input tokens: `613,448`
- Output tokens: `17,912`
- Cache-read tokens: `33,152`
- Requests: `75`
- Total execution time: `674.62s` (`11m 14.62s`)
- Score per 1k tokens: `0.035934`
- Median task score: `0.9700`

Longest tasks:

| Task | Score | Time (s) |
| --- | ---: | ---: |
| `task_02_stock` | `1.0000` | `174.14` |
| `task_16_market_research` | `0.9300` | `138.92` |
| `task_25_access_log_anomaly` | `1.0000` | `46.88` |
| `task_03_blog` | `0.9000` | `43.73` |
| `task_12_skill_search` | `1.0000` | `35.23` |

## Failures And Notes

Primary remaining score losses:

- `task_24_polymarket_briefing` (`0.8250`): market selection still was not
  consistently the top-volume set, and one market/news pairing remained weak.
- `task_03_blog` (`0.9000`): the article was solid, but still graded as more
  conventional than exceptional.
- `task_06_events` (`0.9000`): the table output was useful, but the grader
  still flagged date-specificity risk.
- `task_16_email_triage` (`0.9036`): prioritization was mostly correct, with
  minor deductions on completeness and ordering.

Important recovery:

- `task_22_second_brain` improved from the earlier failed run to `0.9700`,
  indicating that cross-session memory persistence and recall now hold up in
  the benchmark path.

Host validation evidence:

- `./deploy_host.sh` completed successfully twice during this resume cycle and
  reported IPC readiness.
- `./deploy_host.sh --status` confirmed the live host daemon at pid `1016298`
  and the tool executor at pid `1016289`.
- Recent host logs reported `Daemon ready` after the final restart.
- `./deploy_host.sh --test` initially exposed missing test-scope imports for
  `recent_news_selection_score`,
  `format_prediction_market_related_news`, and
  `extract_specific_calendar_dates` in
  `src/tizenclaw/src/core/agent_core.rs`.
- After adding those missing test imports, `./deploy_host.sh --test` passed
  cleanly for the host workspace, mock parity harness, and documentation
  architecture verification.

## Artifacts

- Aggregate JSON:
  [0001_tizenclaw_active-oauth.json](/home/hjhun/samba/github/tizenclaw/.tmp/pinchbench_oauth/results/0001_tizenclaw_active-oauth.json)
- Runner log:
  [latest_full_run.log](/home/hjhun/samba/github/tizenclaw/.tmp/pinchbench_oauth/latest_full_run.log)
- Scratch root:
  [pinchbench_oauth](/home/hjhun/samba/github/tizenclaw/.tmp/pinchbench_oauth)
- Host log:
  [tizenclaw.log](/home/hjhun/.tizenclaw/logs/tizenclaw.log)
