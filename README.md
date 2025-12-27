# process-shepherd

A Windows CLI utility to track what is eating CPU power. Written in Rust.

## Features

- Runs continuously in the background, monitoring all processes
- Tracks CPU utilization per process over the last 60 seconds
- Displays the top 20 processes by average CPU percentage
- Updates display every 2 seconds with real-time data
- Cross-platform compatible (Windows, Linux, macOS)

## Building

### Prerequisites
- Rust toolchain (1.70.0 or later)

### Build Instructions

```bash
# Build release version
cargo build --release

# The binary will be located at:
# target/release/process-shepherd.exe (Windows)
# target/release/process-shepherd (Linux/macOS)
```

### Cross-compilation for Windows (from Linux/macOS)

```bash
# Install Windows target
rustup target add x86_64-pc-windows-gnu

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu
```

## Usage

Simply run the executable:

```bash
# Windows
process-shepherd.exe

# Linux/macOS
./process-shepherd
```

The program will:
1. Start monitoring all running processes
2. Collect CPU usage samples every 2 seconds
3. Display the top 20 processes that have the highest average CPU usage in the last minute
4. Continue running until you press Ctrl+C

### Display Format

The output shows:
- **Process**: Name of the executable
- **PID**: Process ID
- **CPU %**: Average CPU percentage consumed in the tracking window (last 60 seconds)
  - Values are normalized to 0-100% range, regardless of the number of CPU cores
  - A process at 100% is fully utilizing one CPU core
- **Details**: Additional information to distinguish multiple instances of the same process
  - **On Windows**: Shows actual window titles for processes with visible windows (e.g., browser tabs, document names)
  - **On all platforms**: Falls back to command line arguments and memory usage when window titles are not available
  - Shows working directory and memory if no command line arguments are available
  - Helps identify which instance is which when multiple processes share the same name (e.g., multiple Firefox windows)
- **Trend Indicator**: Shows the trend compared to the previous measurement:
  - `↑` - Upward trend (CPU usage increasing)
  - `↓` - Downward trend (CPU usage decreasing)
  - ` ` - Stable (no significant change) or no previous data available

## How It Works

The tool continuously:
1. Samples CPU usage of all processes every 2 seconds
2. Maintains a rolling 60-second window of CPU usage data
3. Calculates average CPU percentage across all samples in the window
4. Normalizes the percentage by dividing by the number of CPU cores to get a 0-100% value
5. Ranks processes by average CPU usage percentage
6. Displays the top 20 CPU consumers

CPU percentage is calculated as the average of all CPU usage samples in the tracking window, divided by the number of CPU cores. This ensures the displayed percentage is in the 0-100% range, where 100% means the process is fully utilizing one CPU core.

## License

See LICENSE file for details.
