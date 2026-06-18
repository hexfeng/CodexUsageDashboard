# Codex Usage Widget

Windows floating widget for Codex local usage limits and token activity.

![Codex Usage Widget](docs/images/codex-usage-widget.png)

The widget reads local Codex session logs from `%USERPROFILE%\.codex\sessions`.
It does not call the OpenAI API, scrape the Usage Dashboard, or read other AI tools.

## v0.1 Scope

- Single floating widget view.
- 12-month token heatmap from local `token_count` events with daily, weekly, and cumulative views.
- 5h and weekly usage bars from local `rate_limits` snapshots.
- Today token total, session count, freshness status, manual refresh, and always-on-top toggle.
- Tauri tray menu: Show, Hide, Refresh, Exit.

## Data Model

The Rust backend persists a small SQLite database under the local app data folder.

- `usage_daily`: daily token aggregates from `payload.info.last_token_usage`.
- `limit_snapshots`: raw 5h/weekly rate limit snapshots.
- `processed_events`: event hashes used to prevent double-counting on repeated scans.
- `app_config`: sessions path and widget settings.

## Requirements

- Node.js 20+.
- npm 10+.
- Rust/Cargo and the Tauri v2 system prerequisites for Windows.

This machine currently has Node/npm available, but `rustc` and `cargo` are not on `PATH`.

## Commands

```powershell
npm install
npm test
npm run build
npm run tauri dev
```

Rust-only checks once Cargo is installed:

```powershell
cd src-tauri
cargo test
cargo build
```

## Notes

- The parser uses `last_token_usage` for daily aggregation so cumulative `total_token_usage` does not get double-counted.
- If no non-null `rate_limits` snapshot is found, the UI shows `Usage limit unavailable` while token activity still works.
- Weekly snapshots with a large usage jump or reset-window movement are marked unusual but still displayed.
