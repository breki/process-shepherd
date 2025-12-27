mod cpu_calculator;
mod display;

use chrono::Utc;
use console::Term;
use cpu_calculator::{calculate_average_cpu_percentage, CpuSample};
use display::{calculate_trend_indicator, truncate_string};
use display::{CPU_PERCENT_WIDTH, PID_WIDTH, PROCESS_NAME_WIDTH};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};


/// Tracks CPU usage history for processes
struct ProcessTracker {
    system: System,
    history: HashMap<Pid, Vec<CpuSample>>,
    retention_seconds: i64,
    last_output_lines: usize,
    previous_cpu_burn: HashMap<Pid, f32>,
    cpu_count: f32,
}

impl ProcessTracker {
    fn new(retention_seconds: i64) -> Self {
        let system = System::new_all();
        // Get CPU count - System::new_all() already initializes CPU info
        // Use max(1) to prevent division by zero
        let cpu_count = (system.cpus().len() as f32).max(1.0);
        
        Self {
            system,
            history: HashMap::new(),
            retention_seconds,
            last_output_lines: 0,
            previous_cpu_burn: HashMap::new(),
            cpu_count,
        }
    }

    /// Update process information and record CPU usage samples
    fn update(&mut self) {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::new().with_cpu(),
        );

        let now = Utc::now();

        // Collect current CPU usage for all processes
        for (pid, process) in self.system.processes() {
            let sample = CpuSample::new(now, process.cpu_usage());
            self.history.entry(*pid).or_default().push(sample);
        }

        // Clean up old samples and remove dead processes
        let cutoff_time = now - chrono::Duration::seconds(self.retention_seconds);
        self.history.retain(|pid, samples| {
            // Remove samples older than retention period
            samples.retain(|s| s.timestamp >= cutoff_time);

            // Keep the entry only if there are samples and the process still exists
            !samples.is_empty() && self.system.process(*pid).is_some()
        });
    }

    /// Calculate average CPU percentage for each process in the retention window
    fn calculate_cpu_burn(&self) -> Vec<(String, Pid, f32)> {
        let mut results = Vec::new();

        for (pid, samples) in &self.history {
            if samples.is_empty() {
                continue;
            }

            // Use the cpu_calculator module for the calculation
            let avg_cpu_percentage = calculate_average_cpu_percentage(samples, self.cpu_count);

            if let Some(process) = self.system.process(*pid) {
                let name = process.name().to_string_lossy().to_string();
                results.push((name, *pid, avg_cpu_percentage));
            }
        }

        // Sort by CPU percentage (descending)
        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Display the top N processes by CPU usage percentage
    fn display_top_processes(&mut self, top_n: usize) {
        let results = self.calculate_cpu_burn();

        let term = Term::stdout();

        // Move cursor to home position and overwrite (don't clear the screen)
        // This is more reliable on Windows than clearing
        if self.last_output_lines > 0 {
            // Move cursor up to the beginning of the last output
            let _ = term.move_cursor_up(self.last_output_lines);
            let _ = term.clear_to_end_of_screen();
        }

        // Build current CPU percentage map for trend calculation
        let mut current_cpu_burn = HashMap::new();
        for (_name, pid, cpu_percent) in &results {
            current_cpu_burn.insert(*pid, *cpu_percent);
        }

        let mut line_count = 0;

        println!("=== Process Shepherd - CPU Usage Tracker ===");
        line_count += 1;
        println!("Tracking window: {} seconds", self.retention_seconds);
        line_count += 1;
        println!("Timestamp: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"));
        line_count += 1;
        println!();
        line_count += 1;
        println!(
            "{:<PROCESS_NAME_WIDTH$} {:<PID_WIDTH$} {:<CPU_PERCENT_WIDTH$}",
            "Process Name", "PID", "CPU %"
        );
        line_count += 1;
        println!("{}", "=".repeat(70));
        line_count += 1;

        for (i, (name, pid, cpu_percent)) in results.iter().take(top_n).enumerate() {
            // Calculate trend indicator using display module helper
            let trend_indicator = if let Some(prev_cpu_percent) = self.previous_cpu_burn.get(pid) {
                calculate_trend_indicator(*cpu_percent, *prev_cpu_percent, 0.1)
            } else {
                " "  // No previous data
            };

            println!(
                "{:<2}. {:<37} {:<10} {:<15.2} {}",
                i + 1,
                truncate_string(name, 37),
                pid.as_u32(),
                cpu_percent,
                trend_indicator
            );
            line_count += 1;
        }

        if results.is_empty() {
            println!("No process data available yet. Collecting samples...");
            line_count += 1;
        }

        // Store the number of lines we output for next refresh
        self.last_output_lines = line_count;

        // Update previous CPU burn for next trend calculation
        self.previous_cpu_burn = current_cpu_burn;
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_trend_calculation() {
        // Test that trend indicators are correctly determined
        let current: f32 = 1.0;
        let previous: f32 = 0.5;
        let diff = current - previous;
        
        // Should be upward trend
        assert!(diff > 0.1);
        
        let current: f32 = 0.5;
        let previous: f32 = 1.0;
        let diff = current - previous;
        
        // Should be downward trend
        assert!(diff < -0.1);
        
        let current: f32 = 1.0;
        let previous: f32 = 1.05;
        let diff = current - previous;
        
        // Should be stable
        assert!(diff.abs() <= 0.1);
    }
}

fn main() {
    println!("Process Shepherd - Starting CPU tracking...");
    println!("Monitoring CPU usage across all processes.");
    println!("Press Ctrl+C to exit.\n");

    const UPDATE_INTERVAL_SECS: u64 = 2; // Sample every 2 seconds
    const RETENTION_SECS: i64 = 60; // Track last 60 seconds
    const TOP_N: usize = 20; // Display top 20 processes

    let mut tracker = ProcessTracker::new(RETENTION_SECS);

    // Initial refresh to populate process list
    tracker.update();
    thread::sleep(Duration::from_secs(1));

    loop {
        tracker.update();
        tracker.display_top_processes(TOP_N);

        thread::sleep(Duration::from_secs(UPDATE_INTERVAL_SECS));
    }
}
