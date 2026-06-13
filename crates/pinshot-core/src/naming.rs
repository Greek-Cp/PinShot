//! Output filename generation (US2).
//!
//! Pure and testable: given a wall-clock timestamp and the names already
//! present in the target folder, produce a non-colliding PNG filename. The
//! shell supplies the current time and the existing names.

/// Builds a `PinShot_YYYY-MM-DD_HH-MM-SS.png` filename, appending `_N` before
/// the extension if that name is already taken (e.g. two saves in one second).
pub fn output_filename(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    existing: &[String],
) -> String {
    let base = format!("PinShot_{year:04}-{month:02}-{day:02}_{hour:02}-{minute:02}-{second:02}");
    let first = format!("{base}.png");
    if !existing.iter().any(|e| e == &first) {
        return first;
    }
    let mut n = 1u32;
    loop {
        let candidate = format!("{base}_{n}.png");
        if !existing.iter().any(|e| e == &candidate) {
            return candidate;
        }
        n += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_timestamp_name() {
        let name = output_filename(2026, 6, 13, 9, 5, 31, &[]);
        assert_eq!(name, "PinShot_2026-06-13_09-05-31.png");
    }

    #[test]
    fn suffixes_on_collision() {
        let existing = vec!["PinShot_2026-06-13_09-05-31.png".to_string()];
        let name = output_filename(2026, 6, 13, 9, 5, 31, &existing);
        assert_eq!(name, "PinShot_2026-06-13_09-05-31_1.png");
    }

    #[test]
    fn finds_next_free_suffix() {
        let existing = vec![
            "PinShot_2026-06-13_09-05-31.png".to_string(),
            "PinShot_2026-06-13_09-05-31_1.png".to_string(),
        ];
        let name = output_filename(2026, 6, 13, 9, 5, 31, &existing);
        assert_eq!(name, "PinShot_2026-06-13_09-05-31_2.png");
    }
}
