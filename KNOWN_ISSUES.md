# Known Issues

## Flaky Test: `test_inter_tribe_predation` in `tests/social_dynamics.rs`

**Status**: Pre-existing issue, not introduced by recent security fixes

**Symptom**: Test assertion fails with "Predator failed to survive or failed to eat prey (Pop: 0)"

**Root Cause**: Under investigation - appears to be a simulation stability issue where both predator and prey die instead of expected predation behavior

**Impact**: Low - Single test in social dynamics test suite

**Resolution**: Requires debugging of inter-tribe predation logic. Test passes on some runs but fails on others, suggesting a timing or sequence dependency.

**Workaround**: Run tests with `--test-threads=1` to reduce flakiness (though this doesn't fully resolve the issue).

**Priority**: LOW - Does not block security fixes or production deployment

---

Last Updated: 2026-02-24
