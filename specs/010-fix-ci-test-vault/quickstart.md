# Quickstart: Fix CI Test Vault Dependency

## Verify the Fix Locally

Before pushing, verify the test-vault clone approach works:

```bash
# Clone the test corpus (same as CI will do)
git clone --depth 1 https://github.com/ashuotaku/Personal-Wiki.git test-vault

# Run the specific test that was failing
cargo test test_graph_edges_from_test_vault -- --nocapture

# Expected: test passes with edge_count > 0

# Clean up
rm -rf test-vault
```

## Verify Pinned Commit

Get the current HEAD of the Personal-Wiki repo for pinning:

```bash
git ls-remote https://github.com/ashuotaku/Personal-Wiki.git HEAD
```

Use the resulting SHA as the `ref` value in the checkout step.

## CI Dogfooding

After pushing the workflow change:

1. The Tests workflow should run with the new clone step
2. Check that the clone step completes without errors
3. Verify `test_graph_edges_from_test_vault` passes
4. Confirm the overall test suite has zero failures

Key indicators of success:
- `cargo test --all-features` exit code is 0
- No "test_graph_edges_from_test_vault ... FAILED" in the test output
- Clone step completes in <30 seconds
