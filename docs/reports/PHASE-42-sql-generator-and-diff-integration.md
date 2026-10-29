# Phase 42 — SQL Generator and per-difference integration

## Scope and audit

The production path passed `SchemaDifference` objects and the target database name to `SqlGenerator::plan_and_generate`. That API had no source snapshot, so it could not generate complete `CREATE TABLE` definitions. The table branch emitted `PlaceholderId` and an implementation comment. Other branches also supplied guessed metadata such as `VARCHAR(100)`, `Id`, `Code`, and `(0)`. Tauri retained the comparison result but did not retain the snapshots required to build real DDL. The differences and risk screens had no operation-specific SQL contract.

## Changes

- SQL generation now receives the source and target snapshots. Table creation uses captured source columns, types, lengths, precision/scale, nullability, identity, collation, defaults, keys, checks, indexes, and foreign keys when supported.
- Missing metadata, computed columns whose persisted state is unavailable, unsupported types, indeterminate diffs, and unsupported transitions produce an explicit unavailable reason. Consolidated export is refused if any diff has no supported operation; it does not export a misleading partial migration.
- The comparison result retains its snapshots in Tauri state. Every current difference gets a generated SQL status, SQL or unavailability reason, and destructive-operation flag. The full script uses those same snapshots and the confirmed target database name.
- Differences and risk alerts expose SQL preview and individual export. Destructive operations require confirmation. Export uses the native save dialog and writes a file; the app does not execute the generated SQL.
- Dashboard alert and plan lists now have bounded scrolling areas. Portuguese labels affected by encoding damage were repaired.
- SQL-file snapshots preserve ASC/DESC key order. Generated index DDL checks whether the named index already exists before creating it.

## Supported operations and limitations

Supported operations depend on exact snapshot metadata and include table/schema creation, column add/drop and supported alterations, primary/unique/default/check/foreign-key/index add/drop, and module definition replacement when the source definition is captured. Destructive operations are flagged. Primary/unique/check/foreign-key/index replacement transitions, generic identity/computed-column alteration, unsupported user-defined types, incomplete table metadata, and several indeterminate object states are refused with a reason. A generated script is a review artifact; preconditions and data compatibility still require review.

## Validation evidence

Rust unit regressions cover complete table DDL, composite keys, defaults, identity, types, collation, index direction/include/filter, and absence of placeholders; missing snapshots are explicitly unavailable. The SQL parser has ASC/DESC regression coverage. The frontend includes SQL mapping and export integration tests, plus the impact dashboard scroll layout.

Validation performed for this change set:

- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace`: passed. The SQL parser and SQL generator regressions ran. Optional SQL Server tests requiring `SCHEMALENS_TEST_*_CONNECTION_STRING` were skipped because those environment variables were not configured; the invalid-endpoint regression ran.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `npm test -- --watch=false`: passed, 19 tests.
- `npm run build`: passed.
- `git diff --check`: passed.

No live SQL Server evidence is claimed by this report. The environment-gated integration tests must run with separate schemas configured to demonstrate live catalog provenance and generated DDL against SQL Server.

## Read-only boundary

The SQL reader continues to read catalog metadata. Tauri has no execution command for migration SQL. Preview and export call the existing file writer only; neither path submits generated DDL to either database.
