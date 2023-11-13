#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum PrintLevel {
    Verbose,
    Normal,
}

#[derive(Debug)]
pub struct Shell {
    level: Option<PrintLevel>,
}

impl Shell {
    pub fn new(level: PrintLevel) -> Self {
        Self { level: Some(level) }
    }

    #[allow(unused)] // useful for tests
    pub fn muted() -> Self {
        Self { level: None }
    }

    pub fn println(&mut self, level: PrintLevel, func: impl FnOnce() -> String) {
        if !self.should_print(level) {
            return;
        }

        println!("{}", func());
    }

    fn should_print(&self, level: PrintLevel) -> bool {
        self.level.map(|lvl| level >= lvl).unwrap_or(false)
    }
}

#[cfg(test)]
mod test {
    use crate::shell::PrintLevel;

    #[test]
    fn test_print_level_compare() {
        assert_eq!(PrintLevel::Verbose, PrintLevel::Verbose);
        assert!(PrintLevel::Verbose < PrintLevel::Normal);
        assert!(PrintLevel::Normal > PrintLevel::Verbose);
        assert_eq!(PrintLevel::Normal, PrintLevel::Normal);
    }
}
