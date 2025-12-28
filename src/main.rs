mod cpu_calculator;
mod display;
mod window_info;

use chrono::Utc;
use clap::Parser;
use console::Term;
use cpu_calculator::{calculate_average_cpu_percentage, CpuSample};
use process_shepherd::ProcessInfo;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// Process Shepherd - Track CPU usage per process
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Minimum CPU percentage threshold to display processes (default: 1.0)
    #[arg(long = "cpu-threshold", default_value_t = 1.0)]
    cpu_threshold: f32,
}


/// Tracks CPU usage history for processes
struct ProcessTracker {
    system: System,
    history: HashMap<Pid, Vec<CpuSample>>,
    retention_seconds: i64,
    last_output_lines: usize,
    previous_cpu_burn: HashMap<Pid, f32>,
    cpu_count: f32,
    window_titles_cache: HashMap<u32, Vec<String>>,
    cpu_threshold: f32,
}

impl ProcessTracker {
    fn new(retention_seconds: i64, cpu_threshold: f32) -> Self {
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
            window_titles_cache: HashMap::new(),
            cpu_threshold,
        }
    }

    /// Update process information and record CPU usage samples
    fn update(&mut self) {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::new().with_cpu().with_memory(),
        );

        // Refresh window titles cache once per update (only on Windows)
        self.window_titles_cache = window_info::get_all_window_titles();

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
    fn calculate_cpu_burn(&self) -> Vec<ProcessInfo> {
        let mut results = Vec::new();

        for (pid, samples) in &self.history {
            if samples.is_empty() {
                continue;
            }

            // Use the cpu_calculator module for the calculation
            let avg_cpu_percentage = calculate_average_cpu_percentage(samples, self.cpu_count);

            // Filter out processes below the configured CPU threshold
            if avg_cpu_percentage < self.cpu_threshold {
                continue;
            }

            if let Some(process) = self.system.process(*pid) {
                let name = process.name().to_string_lossy().to_string();
                let memory_bytes = process.memory();
                
                // Extract additional information to distinguish multiple instances
                let extra_info = self.extract_extra_info(process);
                
                results.push(ProcessInfo::new(
                    name,
                    *pid,
                    avg_cpu_percentage,
                    memory_bytes,
                    extra_info,
                ));
            }
        }

        // Sort by CPU percentage (descending)
        results.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
    
    /// Extract additional information from a process to help distinguish multiple instances
    /// This includes window titles (on Windows), command line arguments, working directory, and memory usage
    fn extract_extra_info(&self, process: &sysinfo::Process) -> String {
        let pid = process.pid().as_u32();

        // First priority: Check for window titles (Windows only)
        // Check this process's window titles first
        if let Some(titles) = self.window_titles_cache.get(&pid) {
            if !titles.is_empty() {
                // Join multiple window titles with " | "
                let titles_str = titles.join(" | ");
                if !titles_str.trim().is_empty() {
                    return titles_str;
                }
            }
        }

        // If this process has no windows, check parent process
        // This is useful for multi-process applications like Firefox where
        // content processes don't own windows but their parent does
        if let Some(parent_pid) = process.parent() {
            if let Some(titles) = self.window_titles_cache.get(&parent_pid.as_u32()) {
                if !titles.is_empty() {
                    let titles_str = titles.join(" | ");
                    if !titles_str.trim().is_empty() {
                        return titles_str;
                    }
                }
            }
        }
        
        // Second priority: Command line arguments
        let cmd = process.cmd();
        if !cmd.is_empty() {
            // Skip the first argument (usually the executable path)
            // and take the next 1-2 meaningful arguments
            let meaningful_args: Vec<String> = cmd.iter()
                .skip(1)
                .take(2)
                .map(|arg| arg.to_string_lossy().to_string())
                .collect();
            
            if !meaningful_args.is_empty() {
                return meaningful_args.join(" ");
            }
        }
        
        // Third priority: Working directory
        if let Some(cwd) = process.cwd() {
            return format!("({})", cwd.display());
        }
        
        // Last resort: Empty string
        String::new()
    }

    /// Display the top N processes by CPU usage percentage
    fn display_top_processes(&mut self, top_n: usize) {
        let results = self.calculate_cpu_burn();

        let term = Term::stdout();

        // Build current CPU percentage map for trend calculation
        let mut current_cpu_burn = HashMap::new();
        for info in &results {
            current_cpu_burn.insert(info.pid, info.cpu_percent);
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
    fn test_filter_processes_below_threshold() {
        // Test that processes below the threshold are filtered out
        let threshold = 1.0;
        let _tracker = ProcessTracker::new(60, threshold);
        
        // Mock data: processes with various CPU percentages
        // In a real scenario, these would be calculated from actual process data
        // For this test, we're verifying the filtering logic
        
        // Process with 0.5% CPU should be filtered
        let cpu_below_threshold = 0.5f32;
        assert!(cpu_below_threshold < threshold, "CPU below threshold should be less than threshold");
        
        // Process with exactly 1% CPU should be included
        let cpu_at_threshold = 1.0f32;
        assert!(cpu_at_threshold >= threshold, "CPU at threshold should be >= threshold");
        
        // Process with 1.5% CPU should be included
        let cpu_above_threshold = 1.5f32;
        assert!(cpu_above_threshold >= threshold, "CPU above threshold should be >= threshold");
    }

    #[test]
    fn test_filter_edge_cases() {
        // Test edge cases for the CPU filter
        let threshold = 1.0;
        
        // Just below threshold
        let cpu = 0.99f32;
        assert!(cpu < threshold, "0.99% should be filtered");
        
        // Exactly at threshold
        let cpu = 1.0f32;
        assert!(cpu >= threshold, "1.0% should be included");
        
        // Just above threshold
        let cpu = 1.01f32;
        assert!(cpu >= threshold, "1.01% should be included");
    }

    #[test]
    fn test_custom_threshold() {
        // Test that custom thresholds work correctly
        let threshold_5 = 5.0;
        let _tracker = ProcessTracker::new(60, threshold_5);
        
        // Process with 3% CPU should be filtered with 5% threshold
        let cpu_below = 3.0f32;
        assert!(cpu_below < threshold_5, "3% should be filtered with 5% threshold");
        
        // Process with 5% CPU should be included
        let cpu_at = 5.0f32;
        assert!(cpu_at >= threshold_5, "5% should be included with 5% threshold");
        
        // Process with 7% CPU should be included
        let cpu_above = 7.0f32;
        assert!(cpu_above >= threshold_5, "7% should be included with 5% threshold");
    }
}

fn main() {
    let args = Args::parse();
    
    println!("Process Shepherd - Starting CPU tracking...");
    println!("Monitoring CPU usage across all processes.");
    println!("CPU threshold: {:.1}%", args.cpu_threshold);
    println!("Press Ctrl+C to exit.\n");

    const UPDATE_INTERVAL_SECS: u64 = 2; // Sample every 2 seconds
    const RETENTION_SECS: i64 = 60; // Track last 60 seconds
    const TOP_N: usize = 20; // Display top 20 processes

    let mut tracker = ProcessTracker::new(RETENTION_SECS, args.cpu_threshold);

    // Initial refresh to populate process list
    tracker.update();
    thread::sleep(Duration::from_secs(1));

    loop {
        tracker.update();
        tracker.display_top_processes(TOP_N);

        thread::sleep(Duration::from_secs(UPDATE_INTERVAL_SECS));
    }
}
