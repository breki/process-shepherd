# Copilot Instructions for process-shepherd

## Project Overview

process-shepherd is a cross-platform (Windows, Linux, macOS) CLI utility written in Rust that tracks CPU usage per process. It continuously monitors all running processes, tracks their CPU utilization over a rolling 60-second window, and displays the top 20 processes by average CPU percentage.

## Tech Stack

- **Language**: Rust (edition 2021, requires Rust 1.70.0 or later)
- **Key Dependencies**:
  - `sysinfo` (0.32): System and process information gathering
  - `chrono` (0.4): Time and date handling for timestamps
  - `console` (0.15): Terminal manipulation and display

## Build and Test Instructions

### Building
```bash
# Debug build
cargo build

# Release build (recommended for performance)
cargo build --release

# Cross-compile for Windows (from Linux/macOS)
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test cpu_calculator
cargo test display
```

### Linting
```bash
# Check for common mistakes
cargo clippy

# Format code
cargo fmt
```

## Project Structure

```
src/
├── main.rs           # Entry point, ProcessTracker implementation, main loop
├── lib.rs            # Library exports
├── cpu_calculator.rs # CPU usage calculation and averaging logic
└── display.rs        # Terminal display formatting and rendering
```

### Module Responsibilities

- **main.rs**: Contains the `ProcessTracker` struct that manages system monitoring, CPU history, and orchestrates updates
- **cpu_calculator.rs**: Pure calculation logic for CPU percentage averaging and normalization
- **display.rs**: Handles all terminal output, formatting, trend indicators, and string truncation

## Coding Conventions

### Testing
- All modules have comprehensive unit tests
- Tests use the `#[cfg(test)]` attribute
- Test functions are named descriptively: `test_<scenario>_<expected_behavior>`
- Use realistic test scenarios (e.g., `test_realistic_dual_core_scenario`)
- Test edge cases: empty inputs, zero/negative values, boundary conditions

### Documentation
- Public functions have doc comments with `///`
- Include `# Arguments`, `# Returns`, and `# Examples` sections where appropriate
- Example code in doc comments should be runnable (use doctest format)

### Error Handling
- Prefer returning 0.0 or safe defaults over panicking
- Use `.max(1.0)` to prevent division by zero (e.g., CPU count)
- Validate inputs at function boundaries

### Code Style
- Use `rustfmt` defaults (4-space indentation)
- Follow Rust naming conventions: `snake_case` for functions/variables, `CamelCase` for types
- Prefer explicit types in function signatures
- Use descriptive variable names

### CPU Percentage Normalization
- Raw CPU usage from `sysinfo` is not normalized (can exceed 100% on multi-core systems)
- ALWAYS normalize by dividing by CPU core count to get 0-100% range
- 100% means one full CPU core is utilized
- Example: 200% raw usage on 4-core system = 50% normalized

### Terminal Display
- Use `console::Term` for terminal manipulation
- Clear previous output by moving cursor up and clearing to end of screen
- Count output lines for proper terminal refresh
- Disable pagers when using git commands: `git --no-pager`

## Common Tasks

### Adding a New CPU Metric
1. Add the metric to `CpuSample` struct if it's time-series data
2. Update calculation logic in `cpu_calculator.rs`
3. Add comprehensive unit tests
4. Update display logic in `display.rs` if UI changes needed
5. Test on multiple platforms if the metric is OS-specific

### Modifying Display Format
1. Update constants in `display.rs` (e.g., `PROCESS_NAME_WIDTH`)
2. Adjust formatting in `display_top_processes` function
3. Update line counting logic to maintain correct terminal refresh
4. Test with long process names and edge cases

### Changing Retention Window
- Modify `RETENTION_SECS` constant in `main.rs`
- Ensure `UPDATE_INTERVAL_SECS` is appropriate for the window size
- Consider memory implications for longer windows

## Cross-Platform Considerations

- Process names may differ across platforms (e.g., ".exe" suffix on Windows)
- Terminal support varies: test display formatting on each platform
- CPU counting should work consistently via `sysinfo::System::cpus()`
- Path separators and line endings are handled by Rust std library

## Dependencies and Security

- Keep dependencies minimal and up-to-date
- `sysinfo` is the core dependency for system monitoring
- Avoid adding dependencies that duplicate stdlib functionality
- Check for security advisories when updating dependencies

## Performance Considerations

- Sample interval (2 seconds) balances responsiveness and CPU overhead
- HashMap storage for process history is efficient for lookup and cleanup
- Sorting by CPU percentage is O(n log n) but n is limited by process count
- Terminal updates should be fast; avoid unnecessary redraws
