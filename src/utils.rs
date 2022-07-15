
/// truncate returns the result of rounding x toward zero to a multiple of m.
/// If m <= 0, Truncate returns x unchanged.
pub fn truncate(x: i64, m: i64) -> i64 {
    if m <= 0 {
        return x
    }
    return x - x % m
}

