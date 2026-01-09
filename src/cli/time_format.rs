/// Time display information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeDisplay {
    /// Elapsed time (seconds)
    pub elapsed: u64,
    /// Total time (seconds)
    pub total: u64,
    /// Percentage (0-100)
    pub percentage: u8,
}

impl TimeDisplay {
    pub fn new(elapsed: u64, total: u64) -> Self {
        let percentage = if total > 0 {
            let p = elapsed.saturating_mul(100) / total;
            p.min(100) as u8
        } else {
            0
        };
        Self {
            elapsed,
            total,
            percentage,
        }
    }

    pub fn format(&self) -> String {
        let e_m = self.elapsed / 60;
        let e_s = self.elapsed % 60;
        let t_m = self.total / 60;
        let t_s = self.total % 60;
        format!(
            "{:02}:{:02}/{:02}:{:02} ({}%)",
            e_m, e_s, t_m, t_s, self.percentage
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_display_new_normal() {
        let display = TimeDisplay::new(323, 1500);
        assert_eq!(display.elapsed, 323);
        assert_eq!(display.total, 1500);
        assert_eq!(display.percentage, 21);
    }

    #[test]
    fn test_time_display_new_zero_total() {
        let display = TimeDisplay::new(100, 0);
        assert_eq!(display.percentage, 0);
    }

    #[test]
    fn test_time_display_new_overflow() {
        let display = TimeDisplay::new(2000, 1500);
        assert_eq!(display.percentage, 100);
    }

    #[test]
    fn test_time_display_format_normal() {
        let display = TimeDisplay::new(323, 1500);
        assert_eq!(display.format(), "05:23/25:00 (21%)");
    }

    #[test]
    fn test_time_display_format_zero() {
        let display = TimeDisplay::new(0, 1500);
        assert_eq!(display.format(), "00:00/25:00 (0%)");
    }

    #[test]
    fn test_time_display_format_complete() {
        let display = TimeDisplay::new(1500, 1500);
        assert_eq!(display.format(), "25:00/25:00 (100%)");
    }

    #[test]
    fn test_time_display_format_padding() {
        let display = TimeDisplay::new(65, 300);
        assert_eq!(display.format(), "01:05/05:00 (21%)");
    }

    #[test]
    fn test_percentage_boundary_values() {
        // 0%
        assert_eq!(TimeDisplay::new(0, 100).percentage, 0);
        // 50%
        assert_eq!(TimeDisplay::new(50, 100).percentage, 50);
        // 99%
        assert_eq!(TimeDisplay::new(99, 100).percentage, 99);
        // 100%
        assert_eq!(TimeDisplay::new(100, 100).percentage, 100);
        // >100% (capped)
        assert_eq!(TimeDisplay::new(150, 100).percentage, 100);
    }
}
