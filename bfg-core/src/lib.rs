//!
//! Holds the core logic and traits for bfg
//!
//!

///# Examples
/// ```
/// let five = 5;
/// assert_eq!(6, bfg_core::add_one(5));
/// ```
pub fn add_one(x: i32) -> i32 {
    x + 1
}

/// Main structure for holding a Bfg
#[derive(Default, Debug, PartialEq)]
pub struct Bfg {
    pub one: i32,
    pub two: i32,
}

impl Bfg {
    /// # Examples
    /// ```
    /// let bfg = bfg_core::Bfg {one: 1, two: 2};
    /// assert_eq!(bfg, bfg_core::Bfg::new());
    /// ```
    pub fn new() -> Bfg {
        Bfg { one: 1, two: 2 }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
