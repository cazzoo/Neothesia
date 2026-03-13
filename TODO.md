# LUMI Hinting Implementation - Active Tasks

## Critical Issue - Song Loop Hinting Not Working

**Status**: SysEx messages reach hardware with correct topologyIndex (0x00), but keyboard remains lit during song playback instead of turning dark and showing hints.

### Root Cause Analysis
- ✅ **FIXED**: topologyIndex was 0x37, now corrected to 0x00
- ✅ **VERIFIED**: SysEx messages reach hardware (seen in logs)
- ❌ **PROBLEM**: Keyboard stays lit in menu mode during song playback
- ❌ **PROBLEM**: No visual hinting effect for upcoming notes

### Required Implementation - Two Distinct LED Control Loops

#### 1. Menu Loop (Currently Working ✅)
- **Purpose**: Apply user settings from LUMI Hardware menu
- **Behavior**: Keyboard shows configured brightness and color mode
- **Location**: Menu scene settings
- **Commands**: `set_color_mode()`, `set_brightness()`

#### 2. Song Loop (Currently Broken ❌)
- **Purpose**: Gameplay hinting and visual feedback
- **Entry behavior**:
  - Clear all LEDs (turn keyboard dark)
  - Apply dim blue hints 2 seconds before notes arrive
  - Keys turn fully bright green when pressed
- **Exit behavior**: Return to menu mode settings
- **Location**: Playing scene
- **Commands**: `clear_all()`, `set_key_dim()` for hints

### Technical Investigation Needed

1. **Why does `clear_all()` not turn off keyboard LEDs?**
   - Compare with reference implementation's LED off command
   - Reference sends: `[0x10, 0x20/0x30, 0x04, 0x00, 0x00, 0x00, 0x7E, 0xFF]` for black color
   - Our implementation sends `(0, 0, 0)` to each key 48-71

2. **Is there a mode switch required?**
   - Menu mode: Color mode (Rainbow/Single/Piano/Night) controls LEDs
   - Song mode: Need manual per-key control - does this require disabling color mode first?

3. **Brightness/mode interaction**
   - Does global brightness setting override per-key colors?
   - Does color mode need to be switched to "manual" for song playback?

### Next Steps

1. Update LUMI_STATUS.md with current findings
2. Update LUMI_PLAN.md with two-loop architecture
3. Investigate LED off command vs `clear_all()` implementation
4. Test if entering song mode requires sending a "manual mode" command
5. Implement proper mode switching between menu and song loops

## References

- Working reference: `.tmp-lumi/lumi-web-control/src/src/lumiSysexLib.js`
- Current implementation: `neothesia/src/lumi_controller.rs`
- Test log: `lumi_final_test.log` (shows SysEx with correct topologyIndex 0x00)
