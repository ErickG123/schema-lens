# PHASE 41 — Risk, compatibility, and change impact

Date: 2026-10-29
Status: IN_PROGRESS — contextual rules and frontend regressions implemented; Rust and live data verification remain unrun.

## Audit findings

The DiffEngine already compared data type, max_length, nullability, collation, PK, FK, UNIQUE, CHECK, DEFAULT, and indexes. It did not compare precision/scale, ignored FK update action/disabled state and CHECK disabled state, and the live reader left every constraint/index collection empty. The previous Angular risk service instead looked for the example object names and returned canned impacts and SQL steps. The current comparison DTO also omitted property name, object type, confidence, and risk assessment.

## Changes

- Added precision/scale differences and included FK schema/update action/disabled state and CHECK disabled state in comparison.
- Live catalog reader now populates PK, FK, UNIQUE, CHECK, DEFAULT, and supported rowstore index metadata. Missing database-wide metadata visibility stops the snapshot.
- Diff DTO carries object type, property, confidence, and risk assessment.
- Impact rules use the actual diff property and A→B values. They explain nullable tightening, a stricter target, required new columns, defaults, type/size/precision, constraints, indexes, and target-only objects.
- Risk severity is kept separate from confidence. Row-level problems are described as conditional because data is not queried.
- New required columns include available default/identity/computed metadata in the evidence. No business fill value is invented.
- Risk UI displays unverified conditions and preconditions, and links an alert to its object in the current ER diagram. The diagram is built from current source and target snapshots and shows identical tables, column types/nullability, primary keys, and catalog-derived foreign-key edges in addition to diff status.
- A SQL execution plan is not synthesized without a snapshot-validated planner. SQL export itself uses the actual latest DiffEngine result and remains manual. Destination-only objects require explicit confirmation before export.

## Tests and evidence

Frontend unit tests cover both nullability directions, required column with and without default, conditional constraint risks, deterministic classification, empty comparisons, no unvalidated execution plan, and snapshot-backed diagram metadata for identical tables and relationships. Latest run: 18 frontend tests passed; Angular production build passed.

No Rust compiler, Cargo, Docker daemon, or SQL Server was available. No live NULL, duplicate, range, orphan, permission, or timeout query was performed. The application does not yet provide optional row-level compatibility scans. Those impacts remain explicitly unverified.

## Remaining limitations

- SQL data verification is not implemented. There are no claims that a target contains NULLs, duplicates, out-of-range values, or orphaned references.
- Index sort direction is not present in the domain model; unsupported SQL Server object types and cross-module dependencies are not covered by the risk view.
- The diagram receives table snapshots, but the frontend impact rules still receive structural differences rather than row-level compatibility scan results. Object inspection still reports indeterminate when detailed definitions are unavailable in its contract.
- `SqlGenerator` output has not been compiled or exercised against live snapshots in this environment. Treat generated SQL as a draft for human review.


