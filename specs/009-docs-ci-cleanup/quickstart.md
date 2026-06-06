# Quickstart: Documentation Cleanup and CI Modernization

## Verify Documentation Accuracy

After changes are applied, verify the documentation is correct:

```bash
# 1. Confirm no stale sqlite-vec references in active docs
grep -n "sqlite-vec\|SQLite/vec" README.md CONTRIBUTING.md
# Expected: Only matches in historical context (Credits section mentioning migration)
# Should NOT show sqlite-vec as the active vector backend

# 2. Confirm MSRV consistency across files
grep "rust-version" Cargo.toml
# Expected: rust-version = "1.75"

grep "1\.75" README.md
# Expected: badge and text references all show 1.75+

# 3. Confirm SQLite prerequisite removed
grep "SQLite" CONTRIBUTING.md
# Expected: No match (or only in context of "no longer required")

# 4. Confirm Cargo.toml has MSRV field
grep "^rust-version" Cargo.toml
# Expected: rust-version = "1.75"
```

## Verify Mermaid Diagram

View Architecture.md in a Mermaid-compatible renderer:

```bash
# Render locally (if mermaid-cli installed)
mmdc -i Architecture.md -o /dev/null

# Or view on GitHub by pushing to remote and checking the rendered markdown
```

Check the Model Download Flow diagram renders without parse errors. Verify other diagrams still render correctly.

## Verify CI Workflows

```bash
# 1. Check for deprecated action references
grep -rn "actions-rs" .github/workflows/
# Expected: No matches

# 2. Check cache version
grep -rn "actions/cache@" .github/workflows/
# Expected: All references use @v4

# 3. Check build matrix
grep -A5 "matrix:" .github/workflows/build.yml
# Expected: Only stable in rust list, no beta/nightly

# 4. Validate YAML syntax
yamllint .github/workflows/*.yml

# 5. Validate action references
actionlint .github/workflows/*.yml
```

## Dogfooding

Push to the feature branch and observe CI results on GitHub:

1. **Test workflow**: Should pass all gates (fmt, clippy, test, audit, deny, MSRV)
2. **Build workflow**: Should build on ubuntu, macos, windows (stable only) in debug and release
3. **Release workflow**: Triggered only on `v*` tags — verify action references are correct by inspection

Key indicators of success:
- Zero deprecation warnings in the Actions UI for any workflow
- `cargo-deny` step completes in <30s on second run (cache hit)
- MSRV check uses `1.75.0` derived from Cargo.toml
