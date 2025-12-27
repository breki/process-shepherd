use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// Represents a CPU usage sample for a process at a specific time
#[derive(Clone)]
struct CpuSample {
    timestamp: DateTime<Utc>,
    cpu_usage: f32,
}

/// Tracks CPU usage history for processes
struct ProcessTracker {
    system: System,
    history: HashMap<Pid, Vec<CpuSample>>,
    retention_seconds: i64,
}

impl ProcessTracker {
    fn new(retention_seconds: i64) -> Self {
        Self {
            system: System::new_all(),
            history: HashMap::new(),
            retention_seconds,
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
            let sample = CpuSample {
                timestamp: now,
                cpu_usage: process.cpu_usage(),
            };
            
            self.history
                .entry(*pid)
                .or_insert_with(Vec::new)
                .push(sample);
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

    /// Calculate total CPU time burned by each process in the retention window
    fn calculate_cpu_burn(&self) -> Vec<(String, Pid, f32)> {
        let mut results = Vec::new();
        
        for (pid, samples) in &self.history {
            if samples.len() < 2 {
                continue;
            }
            
            // Calculate cumulative CPU usage across all samples
            // CPU usage is reported as percentage, so we integrate over time
            let mut total_cpu_seconds = 0.0;
            
            for i in 1..samples.len() {
                let time_delta = (samples[i].timestamp - samples[i - 1].timestamp)
                    .num_milliseconds() as f32 / 1000.0;
                
                // Average CPU usage between samples * time delta
                let avg_cpu = (samples[i].cpu_usage + samples[i - 1].cpu_usage) / 2.0;
                total_cpu_seconds += (avg_cpu / 100.0) * time_delta;
            }
            
            if let Some(process) = self.system.process(*pid) {
                let name = process.name().to_string_lossy().to_string();
                results.push((name, *pid, total_cpu_seconds));
            }
        }
        
        // Sort by CPU time (descending)
        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        results
    }

    /// Display the top N processes by CPU burn
    fn display_top_processes(&self, top_n: usize) {
        let results = self.calculate_cpu_burn();
        
        // Clear screen (Windows-compatible)
        print!("\x1B[2J\x1B[1;1H");
        
        println!("=== Process Shepherd - CPU Usage Tracker ===");
        println!("Tracking window: {} seconds", self.retention_seconds);
        println!("Timestamp: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"));
        println!();
        println!("{:<40} {:<10} {:<15}", "Process Name", "PID", "CPU Time (s)");
        println!("{}", "=".repeat(70));
        
        for (i, (name, pid, cpu_time)) in results.iter().take(top_n).enumerate() {
            println!("{:<2}. {:<37} {:<10} {:<15.2}", 
                i + 1, 
                truncate_string(name, 37),
                pid.as_u32(), 
                cpu_time
            );
        }
        
        if results.is_empty() {
            println!("No process data available yet. Collecting samples...");
        }
    }
}

/// Truncate a string to a maximum length, adding ellipsis if needed
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
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
