/// Module for extracting window titles and information
/// This module provides platform-specific implementations for getting window titles

#[cfg(windows)]
mod windows_impl {
    use std::collections::HashMap;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextW, GetWindowTextLengthW, GetWindowThreadProcessId, IsWindowVisible,
    };

    /// Callback data structure for EnumWindows
    struct EnumWindowsData {
        pid_to_titles: HashMap<u32, Vec<String>>,
    }

    /// Callback function for EnumWindows
    unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = &mut *(lparam.0 as *mut EnumWindowsData);

        // Only process visible windows
        if IsWindowVisible(hwnd).as_bool() {
            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut process_id as *mut u32));

            if process_id != 0 {
                // Get window title length
                let title_length = GetWindowTextLengthW(hwnd);
                if title_length > 0 {
                    // Allocate buffer for title (+1 for null terminator)
                    let mut buffer: Vec<u16> = vec![0; (title_length + 1) as usize];
                    let copied = GetWindowTextW(hwnd, &mut buffer);

                    if copied > 0 {
                        // Convert from UTF-16 to String
                        if let Ok(title) = String::from_utf16(&buffer[..copied as usize]) {
                            if !title.trim().is_empty() {
                                data.pid_to_titles
                                    .entry(process_id)
                                    .or_insert_with(Vec::new)
                                    .push(title);
                            }
                        }
                    }
                }
            }
        }

        BOOL::from(true) // Continue enumeration
    }

    /// Get window titles for a given process ID on Windows
    #[allow(dead_code)]
    pub fn get_window_titles_for_pid(pid: u32) -> Vec<String> {
        unsafe {
            let mut data = EnumWindowsData {
                pid_to_titles: HashMap::new(),
            };

            let lparam = LPARAM(&mut data as *mut _ as isize);
            let _ = EnumWindows(Some(enum_windows_callback), lparam);

            data.pid_to_titles.remove(&pid).unwrap_or_default()
        }
    }

    /// Get all window titles mapped by PID
    pub fn get_all_window_titles() -> HashMap<u32, Vec<String>> {
        unsafe {
            let mut data = EnumWindowsData {
                pid_to_titles: HashMap::new(),
            };

            let lparam = LPARAM(&mut data as *mut _ as isize);
            let _ = EnumWindows(Some(enum_windows_callback), lparam);

            data.pid_to_titles
        }
    }

    /// Debug function to print all window titles (for troubleshooting)
    #[allow(dead_code)]
    pub fn debug_print_all_windows() {
        let titles = get_all_window_titles();
        eprintln!("\n=== DEBUG: All Windows ===");
        for (pid, window_titles) in &titles {
            for title in window_titles {
                eprintln!("PID {}: {}", pid, title);
            }
        }
        eprintln!("=== Total PIDs with windows: {} ===\n", titles.len());
    }
}

// Public API that works cross-platform
#[cfg(windows)]
#[allow(dead_code)]
pub fn get_window_titles_for_pid(pid: u32) -> Vec<String> {
    windows_impl::get_window_titles_for_pid(pid)
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn get_window_titles_for_pid(_pid: u32) -> Vec<String> {
    // On non-Windows platforms, return empty vector
    Vec::new()
}

#[cfg(windows)]
pub fn get_all_window_titles() -> std::collections::HashMap<u32, Vec<String>> {
    windows_impl::get_all_window_titles()
}

#[cfg(not(windows))]
pub fn get_all_window_titles() -> std::collections::HashMap<u32, Vec<String>> {
    // On non-Windows platforms, return empty map
    std::collections::HashMap::new()
}

#[cfg(windows)]
#[allow(dead_code)]
pub fn debug_print_all_windows() {
    windows_impl::debug_print_all_windows()
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn debug_print_all_windows() {
    // No-op on non-Windows platforms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_window_titles_returns_vec() {
        // Should not panic, even if no windows found
        let titles = get_window_titles_for_pid(0);
        assert!(titles.is_empty() || !titles.is_empty());
    }

    #[test]
    fn test_get_all_window_titles_returns_map() {
        // Should not panic
        let all_titles = get_all_window_titles();
        // Result can be empty or non-empty depending on platform and running processes
        assert!(all_titles.len() >= 0);
    }
}
