# Phase 69: Visual Synthesis (ASCII Raytracing)

**Date**: 2026-02-24
**Status**: Design
**Priority**: MEDIUM - Visual polish for improved user experience

---

## Overview

Phase 69 aims to push the visual capabilities of the terminal interface beyond traditional ASCII art by implementing advanced rendering techniques including Signed Distance Field (SDF) rendering and glow effects.

---

## Goals

1. **Enhanced Visual Depth**: Move beyond simple character representation to create more immersive visual feedback
2. **Performance Optimization**: Maintain the current 60 FPS target with new visual effects
3. **Configurable Visuals**: Allow users to tune visual effects based on their preferences
4. **Backward Compatibility**: Ensure that the new rendering doesn't break existing color/symbol schemes

---

## Technical Approach

### 1. SDF (Signed Distance Field) Rendering

**Concept**: Use mathematical distance fields to determine pixel values based on proximity to shapes, enabling smooth gradients and organic shapes.

**Implementation Ideas**:
- Entity Blobs: Draw entities as radial gradients using varying character densities (`░▒▓█`)
- Energy Glow: Entities emit a faint glow based on their energy level
- Distance Attenuation: Objects fade as they move away from the "camera" (or based on world depth)

**Performance Considerations**:
- Precompute character density maps
- Use spatial localization (only render nearby entities with gradients)
- Batch similar operations for cache efficiency

### 2. Glow Effects (Simulated CRT Bloom)

**Concept**: Create a "bloom" effect where bright colors bleed into neighboring cells.

**Implementation**:
- After main rendering pass, identify "bright" cells (high energy, special events)
- Apply color bleeding to adjacent cells with reduced intensity
- Use a decay factor to control the bloom radius

**Example Algorithm**:
```rust
fn apply_glow(grid: &mut Grid) {
    let mut glow_grid = grid.clone();
    for (x, y) in grid.iter_cells() {
        if grid[*x][*y].brightness > 0.8 {
            for neighbor in get_neighbors(*x, *y) {
                let intensity = 0.3 * (1.0 - distance(neighbor, center));
                glow_grid[*x][*y].color = blend_colors(
                    grid[*x][*y].color,
                    grid[neighbor].color,
                    intensity
                );
            }
        }
    }
}
```

### 3. Enhanced Terrain Visualization

**Current**: Single character per terrain type (`,`, `≈`, `▲`, etc.)

**Enhanced**: Use character density to show terrain "richness":
- Plains: `,` (sparse) vs `░` (lush)
- Water: `.` (shallow) vs `≈` (deep)
- Mountains: `▲` (low) vs `▲▲` (peaks)

---

## Design Considerations

### Performance Budget

- Current: ~60 FPS at 10,000 entities
- Goal: Maintain ≥50 FPS with visual enhancements
- Strategy: Level-of-Detail (LOD) rendering based on camera zoom

### User Preferences

**Config Options**:
```toml
[visual]
sdf_rendering = true        # Enable SDF gradients
glow_enabled = true         # Enable bloom effects
glow_intensity = 0.5        # Bloom strength (0.0-1.0)
density_variation = true    # Enable terrain density
color_saturation = 1.0      # Adjust for monochrome terminals
```

### Accessibility

- **Color-Optimized Mode**: Use character density for information when colors are not available
- **High Contrast**: Toggle for low-vision visibility
- **Motion Sickness**: Disable rapid flash/glow effects

---

## Implementation Phases

### Phase 69.1: Character Density Maps (Foundation)
**Time**: ~2 hours
**Tasks**:
- Create density mapping function (distance → character)
- Test density visualization on entities
- Add config option for enable/disable

### Phase 69.2: Entity Glow System
**Time**: ~3 hours
**Tasks**:
- Implement brightness tracking in renderer
- Add post-render glow pass
- Calibrate glow intensity for visual appeal

### Phase 69.3: Terrain Density Variation
**Time**: ~2 hours
**Tasks**:
- Map terrain richness to density
- Update world generation to set density values
- Ensure consistency with fertility system

### Phase 69.4: Performance Optimization
**Time**: ~2 hours
**Tasks**:
- Profile new rendering with 10,000 entities
- Implement LOD (simplified rendering for distant entities)
- Add caching for repeated patterns

### Phase 69.5: Documentation & Testing
**Time**: ~1 hour
**Tasks**:
- Update README with visual features
- Add config examples
- Performance regression tests

**Total Estimated Time**: ~10 hours

---

## Testing Strategy

1. **Unit Tests**: Verify density mapping functions
2. **Integration Tests**: Confirm visual output matches config
3. **Performance Tests**: Ensure FPS doesn't drop below 90% of baseline
4. **A/B Testing**: User feedback on visual quality

---

## Success Criteria

- ✅ Configurable visual effects (on/off, intensity)
- ✅ FPS ≥50 with 10,000 entities (was 60)
- ✅ Backward compatible (no color scheme breaks)
- ✅ Accessible (works in monochrome mode)
- ✅ All existing tests pass

---

## Future Enhancements (Post-69)

1. **3D Perspective**: Isometric projection for terrain depth
2. **Particle Effects**: Birth/death explosion particles
3. **Dynamic Lighting**: Entities cast shadows in darkness
4. **Shader Effects**: Custom ASCII shaders for special events

---

## References

- Current Renderer: `src/app/render.rs`
- TUI Framework: `ratatui` documentation
- SDF Resources: Inigo Quilez's SDF tutorials
- Performance: Existing `perf_gate.rs` benchmarks
