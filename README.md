# process-shepherd

A Windows CLI utility to track what is eating CPU power. Written in Rust.

## Features

- Runs continuously in the background, monitoring all processes
- Tracks CPU utilization per process over the last 60 seconds
- Displays the top 20 processes by cumulative CPU time burned
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
3. Display the top 20 processes that have consumed the most CPU time in the last minute
4. Continue running until you press Ctrl+C

### Display Format

The output shows:
- **Process Name**: Name of the executable
- **PID**: Process ID
- **CPU Time (s)**: Total CPU seconds consumed in the tracking window (last 60 seconds)
- **Trend Indicator**: Shows the trend compared to the previous measurement:
  - `↑` - Upward trend (CPU usage increasing)
  - `↓` - Downward trend (CPU usage decreasing)
  - `→` - Stable (no significant change)
  - ` ` - No previous data available

## How It Works

The tool continuously:
1. Samples CPU usage of all processes every 2 seconds
2. Maintains a rolling 60-second window of CPU usage data
3. Calculates cumulative CPU time by integrating CPU percentage over time
4. Ranks processes by total CPU time burned
5. Displays the top 20 CPU consumers

CPU time is calculated as the average CPU usage percentage between samples multiplied by the time interval, giving you the actual CPU seconds consumed by each process.

## License

See LICENSE file for details.
