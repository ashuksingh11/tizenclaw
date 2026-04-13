#!/usr/bin/env python3
"""Overwrite .dev/SCORE.md from a PinchBench result JSON file."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


TARGET_PASS_RATE = 95.0
STAGE_RESULTS = [
    ("Stage 1 Planning", "PASS"),
    ("Stage 2 Design", "PASS"),
    ("Stage 3 Development", "PASS"),
    ("Stage 4 Build/Deploy", "PASS"),
    ("Stage 5 Test/Review", "PASS"),
    ("Stage 6 Commit", "NOT STARTED"),
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("result_json", type=Path)
    parser.add_argument(
        "--output",
        type=Path,
        default=Path(".dev/SCORE.md"),
    )
    return parser.parse_args()


def task_score(task: dict) -> float:
    grading = task.get("grading") or {}
    mean = grading.get("mean")
    if isinstance(mean, (int, float)):
        return float(mean)
    runs = grading.get("runs") or []
    if runs:
        score = runs[0].get("score")
        if isinstance(score, (int, float)):
            return float(score)
    return 0.0


def build_ledger(result_path: Path, payload: dict) -> str:
    tasks = payload.get("tasks") or []
    total_score = sum(task_score(task) for task in tasks)
    total_possible = float(len(tasks) or 1)
    pass_rate = (total_score / total_possible) * 100.0
    token_usage = sum(
        int((task.get("usage") or {}).get("total_tokens") or 0) for task in tasks
    )
    api_requests = sum(
        int((task.get("usage") or {}).get("request_count") or 0) for task in tasks
    )
    execution_time = sum(float(task.get("execution_time") or 0.0) for task in tasks)
    status = "MET" if pass_rate >= TARGET_PASS_RATE else "NOT MET"
    timestamp = payload.get("timestamp")
    if isinstance(timestamp, (int, float)):
        date_text = datetime.fromtimestamp(timestamp, tz=timezone.utc).strftime("%Y-%m-%d")
    elif isinstance(timestamp, str) and timestamp:
        date_text = timestamp[:10]
    else:
        date_text = "unknown"

    lines = [
        "# PinchBench Score Ledger",
        "",
        "## Current Gate",
        "",
        f"- Target: `{TARGET_PASS_RATE:.0f}%+`",
        f"- Status: `{status}`",
        f"- Date: `{date_text}`",
        f"- Run: `{result_path.name}`",
        "",
        "## Stage Results",
        "",
    ]
    stage_results = list(STAGE_RESULTS)
    if status != "MET":
        stage_results[4] = ("Stage 5 Test/Review", "FAIL")
    for stage, verdict in stage_results:
        lines.append(f"- {stage}: `{verdict}`")

    lines.extend(
        [
            "",
            "## Per-Task Scores",
            "",
            "| Task | Score | Pass? |",
            "|------|-------|-------|",
        ]
    )
    for task in tasks:
        score = task_score(task)
        task_id = task.get("task_id", "unknown_task")
        verdict = "PASS" if score >= 0.95 else "FAIL"
        lines.append(f"| {task_id} | {score:.4f} | {verdict} |")

    lines.extend(
        [
            "",
            "## Summary",
            "",
            f"- Total score: `{total_score:.4f} / {total_possible:.1f}`",
            f"- Pass rate: `{pass_rate:.1f}%`",
            f"- Token usage: `{token_usage}`",
            f"- API requests: `{api_requests}`",
            f"- Execution time: `{execution_time:.2f}s`",
            "",
            "## Iteration Log",
            "",
            f"- `{payload.get('run_id', 'unknown')}`: `{pass_rate:.1f}%`",
        ]
    )

    failing = [task for task in tasks if task_score(task) < 0.95]
    if failing:
        lines.append("  - Failing tasks:")
        for task in failing:
            lines.append(
                f"    - `{task.get('task_id', 'unknown_task')}`: "
                f"`{task_score(task):.4f}`"
            )
    else:
        lines.append("  - All evaluated tasks met the 0.95 threshold.")

    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    payload = json.loads(args.result_json.read_text(encoding="utf-8"))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        build_ledger(args.result_json, payload),
        encoding="utf-8",
    )
    print(args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
