pub enum Shell {
    Noop,
    Stdout,
}

impl Shell {
    pub fn println(&mut self, func: impl FnOnce() -> String) {
        match self {
            Shell::Noop => {}
            Shell::Stdout => println!("{}", func()),
        }
    }
}
