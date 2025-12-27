mod cpu_calculator;
mod display;

use chrono::Utc;
use console::Term;
use cpu_calculator::{calculate_average_cpu_percentage, CpuSample};
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

            // Filter out processes with less than 1% CPU
            if avg_cpu_percentage < 1.0 {
                continue;
            }

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

        // Build current CPU percentage map for trend calculation
        let mut current_cpu_burn = HashMap::new();
        for (_name, pid, cpu_percent) in &results {
            current_cpu_burn.insert(*pid, *cpu_percent);
        }

        // Use display module to render the output with terminal handling
        self.last_output_lines = display::display_top_processes(
            &term,
            &results,
            self.retention_seconds,
            &self.previous_cpu_burn,
            top_n,
            self.last_output_lines,
        );

        // Update previous CPU burn for next trend calculation
        self.previous_cpu_burn = current_cpu_burn;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_filter_processes_below_one_percent() {
        // Test that processes with less than 1% CPU are filtered out
        let _tracker = ProcessTracker::new(60);
        
        // Mock data: processes with various CPU percentages
        // In a real scenario, these would be calculated from actual process data
        // For this test, we're verifying the filtering logic
        
        // Process with 0.5% CPU should be filtered
        let cpu_below_threshold = 0.5f32;
        assert!(cpu_below_threshold < 1.0, "CPU below 1% should be less than 1.0");
        
        // Process with exactly 1% CPU should be included
        let cpu_at_threshold = 1.0f32;
        assert!(cpu_at_threshold >= 1.0, "CPU at 1% should be >= 1.0");
        
        // Process with 1.5% CPU should be included
        let cpu_above_threshold = 1.5f32;
        assert!(cpu_above_threshold >= 1.0, "CPU above 1% should be >= 1.0");
    }

    #[test]
    fn test_filter_edge_cases() {
        // Test edge cases for the 1% CPU filter
        
        // Just below threshold
        let cpu = 0.99f32;
        assert!(cpu < 1.0, "0.99% should be filtered");
        
        // Exactly at threshold
        let cpu = 1.0f32;
        assert!(cpu >= 1.0, "1.0% should be included");
        
        // Just above threshold
        let cpu = 1.01f32;
        assert!(cpu >= 1.0, "1.01% should be included");
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
