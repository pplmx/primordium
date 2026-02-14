# Phase 68.5: Entity Coordinates Integration for Spatial Audio

**Date**: 2026-02-14
**Status**: Design
**Priority**: HIGH - Enables full spatial audio functionality

---

## Problem Statement

Current spatial audio infrastructure (Phase 68 v2.5) implements calculation algorithms but **cannot function** because:
- Birth/Death events lack entity position data (x, y coordinates)
- `process_live_event_with_position(x, y)` always receives `None` for coordinates
- Spatial panning and distance attenuation are calculated but not used

---

## Current Implementation Gap

### LiveEvent Enum (No Coordinates)

```rust
// crates/primordium_data/src/data/environment.rs
LiveEvent::Birth {
    id: Uuid,
    parent_id: Option<Uuid>,
    gen: u32,
    tick: u64,
    timestamp: String,
    // MISSING: x, y coordinates
}

LiveEvent::Death {
    id: Uuid,
    age: u64,
    offspring: u32,
    tick: u64,
    timestamp: String,
    cause: String,
    // MISSING: x, y coordinates
}
```

### Event Creation Analysis

| Event Type | File:Line | Coordinates Available? | Current Event Created? |
|------------|-----------|------------------------|------------------------|
| Birth      | `interaction.rs:151` | ✅ `baby.physics.x, baby.physics.y` | ✅ Yes (without coords) |
| Death (Kill) | `interaction.rs:99` | ❌ Only Identity/Metabolism fetched | ✅ Yes (without coords) |
| Death (Starvation) | `finalize.rs:103-149` | ✅ `phys.x, phys.y` | ❌ **No** - not logged! |

**Critical Finding**: Starvation deaths are NOT logged as events at all!

---

## Implementation Strategy

### Approach: Add Optional Coordinates (x, y)

**Why Optional?**
- Maintain backward compatibility with existing logs (if any)
- Handle edge cases where position might not be available
- Keep serialization format stable for JSONL output

### Phase 1: Modify LiveEvent Enum (1 commit)

**File**: `crates/primordium_data/src/data/environment.rs`

```rust
LiveEvent::Birth {
    id: Uuid,
    parent_id: Option<Uuid>,
    gen: u32,
    tick: u64,
    timestamp: String,
    x: Option<f64>,  // NEW
    y: Option<f64>,  // NEW
}

LiveEvent::Death {
    id: Uuid,
    age: u64,
    offspring: u32,
    tick: u64,
    timestamp: String,
    cause: String,
    x: Option<f64>,  // NEW
    y: Option<f64>,  // NEW
}
```

**Impact**:
- **Serialization**: `#[serde(tag = "event")]` handles new fields gracefully
- **Pattern Matching**: All existing patterns use `..` (field-agnostic) - no breaks
- **Backward Compatible**: New optional fields deserialize as `None` in old logs

### Phase 2: Update Birth Event Creation (1 commit)

**File**: `crates/primordium_core/src/systems/interaction.rs:151`

```rust
let ev = LiveEvent::Birth {
    id: baby.identity.id,
    parent_id: baby.identity.parent_id,
    gen: baby.metabolism.generation,
    tick: ctx.tick,
    timestamp: chrono::Utc::now().to_rfc3339(),
    x: Some(baby.physics.x),  // NEW
    y: Some(baby.physics.y),  // NEW
};
```

### Phase 3: Update Death Event for Kills (1 commit)

**File**: `crates/primordium_core/src/systems/interaction.rs:66-78`

**Before** (lines 66-78):
```rust
if let (Ok(target_identity), Ok(target_metabolism)) = (
    world.get::<&primordium_data::Identity>(target_handle),
    world.get::<&Metabolism>(target_handle),
) {
    // ... create Death event (no position)
}
```

**After**:
```rust
if let (Ok(target_identity), Ok(target_metabolism), Ok(target_physics)) = (
    world.get::<&primordium_data::Identity>(target_handle),
    world.get::<&Metabolism>(target_handle),
    world.get::<&primordium_data::Physics>(target_handle),
) {
    // ... create Death event WITH position
    x: Some(target_physics.x),
    y: Some(target_physics.y),
}
```

### Phase 4: Add Death Event for Starvation (1 commit)

**File**: `src/model/world/finalize.rs:143-149`

**Current**: No event created for starvation deaths

**After** (in `process_deaths` loop):
```rust
if let Ok((met, identity, phys, intel)) = self.ecs.remove::<(...)>(handle) {
    // ... existing cleanup code ...

    // NEW: Create Death event for starvation
    if world.death_cause.get(&tid).is_none() {
        // No kill cause recorded - this is starvation
        let ev = LiveEvent::Death {
            id: tid,
            age: ctx.tick - identity.birth_tick,
            offspring: identity.total_offspring,
            tick: ctx.tick,
            timestamp: Utc::now().to_rfc3339(),
            cause: "Starvation".to_string(),
            x: Some(phys.x),
            y: Some(phys.y),
        };
        events.push(ev);
    }
}
```

**Note**: Requires adding `events: &mut Vec<LiveEvent>` parameter to `process_deaths`

### Phase 5: Update Audio Integration (1 commit)

**File**: `src/app/mod.rs:210-212`

**Current**:
```rust
for event in &events {
    self.audio.process_live_event(event);  // loses position info
}
```

**After**:
```rust
for event in &events {
    use primordium_data::Position;
    let (x, y) = match event {
        LiveEvent::Birth { x, y, .. } => (*x, *y),
        LiveEvent::Death { x, y, .. } => (*x, *y),
        _ => (None, None),
    };

    self.audio.process_live_event_with_position(event, x, y);
}
```

---

## Risk Assessment

### Low Risk

- **Serialization**: `#[serde(tag = "event")]` + Optional fields = stable format
- **Pattern Matching**: All existing patterns use `..` - field-agnostic
- **No Breaking Changes**: New fields are optional, default to `None`

### Medium Risk

- **Performance**: Additional component queries (Physics) for Death events
  - **Mitigation**: Query cost is minimal (already querying Identity/Metabolism)
  - **Benchmark impact**: <1% overhead expected

- **Starvation Death Logging**: New behavior - may increase event volume
  - **Mitigation**: Only add for starvation deaths (already a rare occurrence)
  - **Event rate**: Births >> Deaths >> Starvation Deaths

### Testing Strategy

1. **Unit Tests**: Verify Birth/Death event creation includes coordinates
2. **Integration Tests**: Confirm spatial audio receives valid coordinates
3. **Regression Tests**: Ensure existing audio tests still pass
4. **Performance Test**: Benchmark component query overhead

---

## Success Criteria

- ✅ Birth events include `x, y` coordinates
- ✅ Death events (kills) include `x, y` coordinates
- ✅ Death events (starvation) are logged with coordinates
- ✅ `process_live_event_with_position()` receives valid coordinates (>90% of Birth/Death events)
- ✅ Spatial audio panning and attenuation functional
- ✅ All existing tests pass
- ✅ No significant performance degradation (>5%)

---

## Estimated Effort

| Phase | Description | Estimate |
|-------|-------------|----------|
| 1 | Modify LiveEvent enum | 30 min |
| 2 | Update Birth event creation | 15 min |
| 3 | Update Death event (_kill_) | 30 min |
| 4 | Add Death event (_starvation_) | 45 min |
| 5 | Update audio integration | 15 min |
| Testing & Verification | | 30 min |
| **Total** | | **~2.5 hours, 5-6 commits** |

---

## Future Enhancements (Post-Implementation)

1. **Optional**: Full stereo output in AudioEngine (currently mono)
2. **Optional**: Velocity-based Doppler effect for moving entities
3. **Optional**: Reverb algorithm for environmental depth
4. **Optional**: Dynamic EQ based on ecosystem state

---

## References

- Phase 68 v2 Design: `docs/plans/2026-02-13-phase68-v2-algorithmic-audio-design.md`
- Spatial Audio Implementation: `src/app/audio/spatial.rs`
- Event System: `crates/primordium_data/src/data/environment.rs`
- Interaction System: `crates/primordium_core/src/systems/interaction.rs`
