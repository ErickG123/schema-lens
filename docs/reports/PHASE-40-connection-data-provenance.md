# PHASE 40 — Connection data provenance

Date: 2026-10-29
Status: IN_PROGRESS — root cause and frontend corrections are implemented; Rust and live SQL Server validation are not available in this environment.

## Confirmed root cause

The production comparison did not call the existing SQL Server reader. `src-tauri/src/main.rs::run_comparison` ignored both connection payloads and returned four constant differences copied from the Docker sample. `frontend/src/app/app.component.ts::runComparison` replaced an empty response (including the `{}` returned by `TauriIpcService` outside Tauri) with the same sample. The Angular app also started with Docker server/database profiles and a sample credential. Independently, the ER diagram seeded four fixed tables, local history seeded a fake completed run when storage was empty, and inspection/risk services returned fixture-specific details.

This explains why selecting other endpoints could still show the familiar Docker objects. It was not a SQL Server catalog cache issue: the desktop comparison command bypassed the reader entirely.

## Changes

- Tauri now parses the two submitted profiles independently, opens SQL Server connections, reads two snapshots, runs `DiffEngine`, returns the actual differences and effective database identity, and keeps the latest run for SQL export.
- The catalog reader confirms database-wide `VIEW DEFINITION` or `db_owner` before reporting completeness, returns an error when that cannot be confirmed, and reads schemas, tables, columns, keys, foreign keys, defaults, checks, and supported indexes from catalog views. It aborts on query failures instead of returning a partial snapshot.
- The frontend starts with blank profiles and TLS validation enabled. Editing connection fields invalidates the previous test identity. A comparison tests both profiles, asks before continuing when the effective database differs from the selected database, clears prior results before the run, and refuses empty/error IPC fallbacks.
- Empty history remains empty. The diagram receives both current table snapshots and actual differences, so it can show identical tables and catalog-derived foreign-key edges without fixture nodes. Risk and inspection services no longer invent the Docker example's objects.
- SQL export uses the latest successful backend comparison and target database. It remains an export-only operation. Destination-only objects require explicit review before export.
- Added environment-configured SQL Server provenance, empty database, and unavailable endpoint integration regressions. They were not executed here.

## Evidence collected

- Repository search found the hardcoded production comparison in `src-tauri/src/main.rs`, the frontend empty-response fixture in `app.component.ts`, startup profiles there, fixed diagram nodes in `diagram-builder.service.ts`, seeded history in `comparison-history.service.ts`, plus fixture-specific impact and inspection logic.
- `npm test -- --watch=false`: 18 frontend tests passed, including a regression for identical snapshot tables, primary keys, and a foreign-key relationship.
- `npm run build`: Angular production build passed after the snapshot-backed diagram integration.
- Rust toolchain (`cargo`, `rustfmt`) and Docker daemon are absent. No SQL Server database was reached; no live database provenance claim is made.

## Remaining acceptance work

Rust formatting/build/tests, SQL Server integration with deliberately distinct schemas, empty database, permission denial, effective database mismatch, rapid concurrent runs, and a second run after history restoration remain unverified. The reader has no data compatibility scans; impact alerts explicitly mark row-level conditions unverified. The ER diagram now consumes the current snapshots; object inspection still reports indeterminate when snapshot-level detail is unavailable in its contract.


