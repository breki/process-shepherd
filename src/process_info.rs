use sysinfo::Pid;

/// Information about a process including CPU usage and distinguishing details
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: Pid,
    pub cpu_percent: f32,
    pub extra_info: String,
}

impl ProcessInfo {
    pub fn new(name: String, pid: Pid, cpu_percent: f32, extra_info: String) -> Self {
        Self {
            name,
            pid,
            cpu_percent,
            extra_info,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysinfo::Pid;

    #[test]
    fn test_process_info_creation() {
        let info = ProcessInfo::new(
            "test.exe".to_string(),
            Pid::from_u32(1234),
            50.5,
            "extra details".to_string(),
        );
        
        assert_eq!(info.name, "test.exe");
        assert_eq!(info.pid.as_u32(), 1234);
        assert_eq!(info.cpu_percent, 50.5);
        assert_eq!(info.extra_info, "extra details");
    }

    #[test]
    fn test_process_info_empty_extra_info() {
        let info = ProcessInfo::new(
            "simple.exe".to_string(),
            Pid::from_u32(5678),
            25.0,
            String::new(),
        );
        
        assert_eq!(info.extra_info, "");
    }

    #[test]
    fn test_process_info_long_extra_info() {
        let long_info = "this is a very long command line with many arguments".to_string();
        let info = ProcessInfo::new(
            "app.exe".to_string(),
            Pid::from_u32(9999),
            75.5,
            long_info.clone(),
        );
        
        assert_eq!(info.extra_info, long_info);
    }
}
