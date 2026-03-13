# LUMI Integration Status & Current Investigation

## Latest Discovery: topologyIndex Bug FIXED ✅

**Issue**: SysEx commands were using `topologyIndex = 0x37` instead of `0x00`, causing hardware to reject all commands.

**Root Cause**: `LumiController::send_sysex_command()` was duplicating logic instead of using the corrected helper function.

**Fix Applied** (2025-03-11 01:05):
- Changed line 150 in `lumi_controller.rs` from `msg.push(0x37)` to `msg.push(0x00)`
- Rebuilt release binary
- **Verification**: Log shows `[F0, 00, 21, 10, 77, 00, ...]` - byte 5 is now 0x00 ✅

## Current Status

### What Works ✅
1. **Wait Mode Sound Triggering** - Keys play sound immediately when pressed
2. **LUMI Settings in Menu** - Brightness and color mode changes work from settings menu
3. **SysEx Message Delivery** - Messages reach hardware with correct format
4. **topologyIndex** - Now correctly set to 0x00

### What Doesn't Work ❌
1. **Song Loop Hinting** - Keyboard remains lit during playback, doesn't show hints
2. **LED Clearing** - `clear_all()` at song start doesn't turn keyboard dark

## Architecture Problem Identified

### Current Flaw: Single Control Path
Both menu settings and song hinting use the same LED control path without mode switching:

```
Menu Scene: set_color_mode() → set_brightness() → Keyboard shows settings
     ↓ (user clicks Play)
Playing Scene: clear_all() → set_key_dim() hints → Keyboard ignores?
```

### Required: Two Distinct Control Loops

**Menu Loop (Settings Mode)**:
- Purpose: Display user preferences
- Method: Color mode (Rainbow/Single/Piano/Night) + Brightness percentage
- Control: Hardware's built-in LED patterns
- Status: ✅ **WORKING**

**Song Loop (Manual Mode)**:
- Purpose: Gameplay guidance
- Entry: Clear all LEDs → Switch to manual per-key control
- Method: `set_key_dim()` for hints, `set_key_color()` for pressed notes
- Exit: Return to settings mode
- Status: ❌ **NOT WORKING**

## Technical Analysis

### Reference Implementation (lumi-web-control)

**LED Off Command**:
```javascript
// From lumiSysexLib.js getColor() with #000000
Payload: [0x10, 0x20/0x30, 0x04, 0x00, 0x00, 0x00, 0x7E, 0xFF]
```

**Our clear_all()**:
```rust
// Sends (0, 0, 0) to each key 48-71
fn clear_key(note) {
    self.set_key_color(note, 0, 0, 0);  // Black
}
```

**Question**: Are these equivalent? Or does LUMI need a specific "clear all" command?

### Hypothesis: Mode Switching Required

**Suspected Issue**: Color mode (Night/Rainbow/etc.) may need to be disabled before manual per-key control works.

**Test**:
1. Send `set_color_mode(3)` (Night mode) - works in menu ✅
2. Try `set_key_color()` during playback - ignored? ❌
3. **Missing step**: Need to exit color mode before per-key control?

### Investigation Plan

1. **Compare byte sequences**: Our `clear_all()` vs reference's LED off command
2. **Test mode switching**: Does `set_color_mode()` need to be disabled for manual control?
3. **Check for "manual mode"**: Reference implementation may have a command to switch to direct LED control
4. **Verify timing**: Are commands being sent at the right time in playback loop?

## Log Evidence

**Correct topologyIndex** (lumi_final_test.log line 14505+):
```
[DEBUG] Sending MIDI SysEx: [F0, 00, 21, 10, 77, 00, 10, 40, 62, 00, 00, 00, 00, 00, 7E, F7]
                                           ^^-- NOW 0x00 (was 0x37)
```

**Commands Being Sent**:
- Initialization: Brightness and color mode commands ✅
- During playback: Hinting commands with dim blue (0, 100, 255) ✅
- **But**: Keyboard shows no visual response to hints ❌

## Remaining Work

### Immediate Priority
1. ✅ FIX: topologyIndex (COMPLETED)
2. ❌ DIAGNOSE: Why hints don't appear despite correct SysEx
3. ❌ IMPLEMENT: Proper mode switching between menu and song
4. ❌ VERIFY: LED clearing turns keyboard dark at song start

### Architecture Changes Needed
- **Entry to song mode**: Clear all LEDs, switch to manual control
- **During song**: Per-key hinting with `set_key_dim()`
- **Exit from song**: Restore color mode and brightness settings
- **Two helper functions**:
  - `apply_menu_settings()` - for menu scene
  - `init_song_mode()` - for playing scene entry

## Files Modified This Session

1. **lumi_controller.rs line 150**: Changed `msg.push(0x37)` to `msg.push(0x00)` ✅
2. **TODO.md**: Created with current investigation status ✅
3. **LUMI_STATUS.md**: Updated with topologyIndex fix discovery ✅

## Testing Commands

```bash
# Rebuild with topologyIndex fix
cargo build --release

# Run with debug logging
RUST_LOG=debug ./target/release/neothesia 2>&1 | tee lumi_test.log

# Verify SysEx format
grep "Sending MIDI SysEx" lumi_test.log | head -5

# Expected: [F0, 00, 21, 10, 77, 00, ...]  (byte 5 = 0x00)
```

## Next Session Priorities

1. Compare `clear_all()` byte output with reference LED off command
2. Investigate if color mode needs to be disabled for manual control
3. Test hypothesis: Send specific "enter manual mode" command at song start
4. Implement two-loop architecture with proper mode switching
