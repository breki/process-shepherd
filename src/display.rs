use chrono::Utc;
use console::Term;
use sysinfo::Pid;
use std::collections::HashMap;

// Display formatting constants
pub const PROCESS_NAME_WIDTH: usize = 40;
pub const PID_WIDTH: usize = 10;
pub const CPU_PERCENT_WIDTH: usize = 18;
pub const DISPLAY_SEPARATOR_WIDTH: usize = 73;

/// Truncate a string to a maximum length, adding ellipsis if needed
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len < 3 {
        s.chars().take(max_len).collect()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Calculate trend indicator based on current and previous CPU percentages
/// 
/// # Arguments
/// * `current` - Current CPU percentage
/// * `previous` - Previous CPU percentage
/// * `threshold` - Minimum difference to consider a trend (default: 0.1)
/// 
/// # Returns
/// Trend indicator: "↑" (up), "↓" (down), " " (stable)
pub fn calculate_trend_indicator(current: f32, previous: f32, threshold: f32) -> &'static str {
    let diff = current - previous;
    if diff > threshold {
        "↑"  // Upward trend
    } else if diff < -threshold {
        "↓"  // Downward trend
    } else {
        " "  // Stable/no change
    }
}

/// Display the top N processes by CPU usage with improved terminal handling
///
/// # Arguments
/// * `term` - Terminal reference for cursor control
/// * `results` - Vector of (process_name, pid, cpu_percentage) tuples sorted by CPU usage
/// * `retention_seconds` - Tracking window size in seconds
/// * `previous_cpu_burn` - Map of previous CPU percentages for trend calculation
/// * `top_n` - Number of top processes to display
/// * `last_output_lines` - Number of lines from the previous output (for clearing)
///
/// # Returns
/// The number of lines output (to be used for next refresh)
pub fn display_top_processes(
    term: &Term,
    results: &[(String, Pid, f32)],
    retention_seconds: i64,
    previous_cpu_burn: &HashMap<Pid, f32>,
    top_n: usize,
    last_output_lines: usize,
) -> usize {
    // Move cursor to home position and overwrite (don't clear the screen)
    // This is more reliable on Windows than clearing
    if last_output_lines > 0 {
        // Move cursor up to the beginning of the last output
        let _ = term.move_cursor_up(last_output_lines);
        let _ = term.clear_to_end_of_screen();
    }

    let mut line_count = 0;

    println!("=== Process Shepherd - CPU Usage Tracker ===");
    line_count += 1;
    println!("Tracking window: {} seconds", retention_seconds);
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
    println!("{}", "=".repeat(DISPLAY_SEPARATOR_WIDTH));
    line_count += 1;

    for (i, (name, pid, cpu_percent)) in results.iter().take(top_n).enumerate() {
        // Calculate trend indicator
        let trend_indicator = if let Some(prev_cpu_percent) = previous_cpu_burn.get(pid) {
            calculate_trend_indicator(*cpu_percent, *prev_cpu_percent, 0.1)
        } else {
            " "  // No previous data
        };

        println!(
            "{:>2} {:<37} {:<PID_WIDTH$} {:>6.2}  {}",
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

    line_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_string_short() {
        assert_eq!(truncate_string("short", 10), "short");
    }

    #[test]
    fn test_truncate_string_exact_length() {
        assert_eq!(truncate_string("exactly10!", 10), "exactly10!");
    }

    #[test]
    fn test_truncate_string_long() {
        assert_eq!(truncate_string("this is a very long string", 10), "this is...");
    }

    #[test]
    fn test_truncate_string_edge_case_small() {
        assert_eq!(truncate_string("abc", 3), "abc");
        assert_eq!(truncate_string("abcd", 3), "...");
    }

    #[test]
    fn test_truncate_string_very_small() {
        assert_eq!(truncate_string("abcd", 2), "ab");
        assert_eq!(truncate_string("a", 1), "a");
    }

    #[test]
    fn test_trend_indicator_upward() {
        let indicator = calculate_trend_indicator(1.5, 1.0, 0.1);
        assert_eq!(indicator, "↑");
    }

    #[test]
    fn test_trend_indicator_downward() {
        let indicator = calculate_trend_indicator(1.0, 1.5, 0.1);
        assert_eq!(indicator, "↓");
    }

    #[test]
    fn test_trend_indicator_stable() {
        let indicator = calculate_trend_indicator(1.0, 1.05, 0.1);
        assert_eq!(indicator, " ");
    }

    #[test]
    fn test_trend_indicator_at_threshold() {
        // Small change should be stable (well within threshold)
        let indicator = calculate_trend_indicator(1.05, 1.0, 0.1);
        assert_eq!(indicator, " ");
        
        // Clearly over threshold should be upward
        let indicator = calculate_trend_indicator(1.5, 1.0, 0.1);
        assert_eq!(indicator, "↑");
        
        // Small negative change should be stable
        let indicator = calculate_trend_indicator(0.95, 1.0, 0.1);
        assert_eq!(indicator, " ");
        
        // Clearly below threshold should be downward
        let indicator = calculate_trend_indicator(0.5, 1.0, 0.1);
        assert_eq!(indicator, "↓");
    }

    #[test]
    fn test_trend_indicator_negative_change() {
        let indicator = calculate_trend_indicator(0.5, 1.0, 0.1);
        assert_eq!(indicator, "↓");
    }

    #[test]
    fn test_trend_indicator_custom_threshold() {
        // With threshold of 1.0, a change of 0.5 should be stable
        let indicator = calculate_trend_indicator(1.5, 1.0, 1.0);
        assert_eq!(indicator, " ");
        
        // With threshold of 1.0, a change of 1.5 should be upward
        let indicator = calculate_trend_indicator(2.5, 1.0, 1.0);
        assert_eq!(indicator, "↑");
    }
}
