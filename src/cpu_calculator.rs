use chrono::{DateTime, Utc};

/// Represents a CPU usage sample for a process at a specific time
#[derive(Clone, Debug, PartialEq)]
pub struct CpuSample {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,
}

impl CpuSample {
    pub fn new(timestamp: DateTime<Utc>, cpu_usage: f32) -> Self {
        Self {
            timestamp,
            cpu_usage,
        }
    }
}

/// Calculate average CPU percentage from a set of samples
/// 
/// # Arguments
/// * `samples` - Vector of CPU usage samples
/// * `cpu_count` - Number of CPU cores to normalize against
/// 
/// # Returns
/// Average CPU percentage normalized to 0-100% range where 100% = one full CPU core
/// 
/// # Examples
/// ```
/// use chrono::Utc;
/// use process_shepherd::cpu_calculator::{CpuSample, calculate_average_cpu_percentage};
/// 
/// let now = Utc::now();
/// let samples = vec![
///     CpuSample::new(now, 50.0),
///     CpuSample::new(now, 100.0),
/// ];
/// 
/// // With 4 cores, average (75.0) / 4 = 18.75%
/// let result = calculate_average_cpu_percentage(&samples, 4.0);
/// assert_eq!(result, 18.75);
/// ```
pub fn calculate_average_cpu_percentage(samples: &[CpuSample], cpu_count: f32) -> f32 {
    if samples.is_empty() || cpu_count <= 0.0 {
        return 0.0;
    }

    let total_cpu_usage: f32 = samples.iter().map(|s| s.cpu_usage).sum();
    let average = total_cpu_usage / samples.len() as f32;
    
    // Normalize by CPU count to get 0-100% range
    average / cpu_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_sample_single_core() {
        let now = Utc::now();
        let samples = vec![CpuSample::new(now, 100.0)];
        
        // 100% usage on single core system = 100%
        let result = calculate_average_cpu_percentage(&samples, 1.0);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_single_sample_multi_core() {
        let now = Utc::now();
        let samples = vec![CpuSample::new(now, 200.0)];
        
        // 200% usage (2 cores fully used) on 4 core system = 50%
        let result = calculate_average_cpu_percentage(&samples, 4.0);
        assert_eq!(result, 50.0);
    }

    #[test]
    fn test_multiple_samples_averaging() {
        let now = Utc::now();
        let samples = vec![
            CpuSample::new(now, 100.0),
            CpuSample::new(now, 200.0),
            CpuSample::new(now, 300.0),
        ];
        
        // Average: (100 + 200 + 300) / 3 = 200
        // On 4 cores: 200 / 4 = 50%
        let result = calculate_average_cpu_percentage(&samples, 4.0);
        assert_eq!(result, 50.0);
    }

    #[test]
    fn test_empty_samples() {
        let samples: Vec<CpuSample> = vec![];
        let result = calculate_average_cpu_percentage(&samples, 4.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_zero_cpu_count() {
        let now = Utc::now();
        let samples = vec![CpuSample::new(now, 100.0)];
        
        // Should return 0 to avoid division by zero
        let result = calculate_average_cpu_percentage(&samples, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_negative_cpu_count() {
        let now = Utc::now();
        let samples = vec![CpuSample::new(now, 100.0)];
        
        // Should return 0 for invalid CPU count
        let result = calculate_average_cpu_percentage(&samples, -1.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_realistic_dual_core_scenario() {
        let now = Utc::now();
        // Simulating a process using ~75% of one core on a dual-core system
        let samples = vec![
            CpuSample::new(now, 150.0),  // 150% total (75% normalized for 2 cores)
            CpuSample::new(now, 150.0),
            CpuSample::new(now, 150.0),
        ];
        
        // Average: 150, normalized: 150/2 = 75%
        let result = calculate_average_cpu_percentage(&samples, 2.0);
        assert_eq!(result, 75.0);
    }

    #[test]
    fn test_realistic_quad_core_scenario() {
        let now = Utc::now();
        // Process fully utilizing 2 out of 4 cores
        let samples = vec![
            CpuSample::new(now, 200.0),
            CpuSample::new(now, 200.0),
        ];
        
        // Average: 200, normalized: 200/4 = 50%
        let result = calculate_average_cpu_percentage(&samples, 4.0);
        assert_eq!(result, 50.0);
    }

    #[test]
    fn test_low_cpu_usage() {
        let now = Utc::now();
        // Very light CPU usage
        let samples = vec![
            CpuSample::new(now, 5.0),
            CpuSample::new(now, 3.0),
            CpuSample::new(now, 7.0),
        ];
        
        // Average: 5, normalized on 4 cores: 5/4 = 1.25%
        let result = calculate_average_cpu_percentage(&samples, 4.0);
        assert_eq!(result, 1.25);
    }

    #[test]
    fn test_fractional_cpu_values() {
        let now = Utc::now();
        let samples = vec![
            CpuSample::new(now, 12.5),
            CpuSample::new(now, 37.5),
        ];
        
        // Average: 25, normalized on 2 cores: 25/2 = 12.5%
        let result = calculate_average_cpu_percentage(&samples, 2.0);
        assert_eq!(result, 12.5);
    }

    #[test]
    fn test_eight_core_heavy_usage() {
        let now = Utc::now();
        // Process using 6 out of 8 cores fully
        let samples = vec![
            CpuSample::new(now, 600.0),
            CpuSample::new(now, 600.0),
        ];
        
        // Average: 600, normalized on 8 cores: 600/8 = 75%
        let result = calculate_average_cpu_percentage(&samples, 8.0);
        assert_eq!(result, 75.0);
    }

    #[test]
    fn test_cpu_sample_creation() {
        let now = Utc::now();
        let sample = CpuSample::new(now, 42.5);
        
        assert_eq!(sample.timestamp, now);
        assert_eq!(sample.cpu_usage, 42.5);
    }

    #[test]
    fn test_cpu_sample_clone() {
        let now = Utc::now();
        let sample1 = CpuSample::new(now, 50.0);
        let sample2 = sample1.clone();
        
        assert_eq!(sample1, sample2);
    }
}
