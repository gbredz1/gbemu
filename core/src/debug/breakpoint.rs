use std::collections::HashSet;

#[derive(Default)]
pub struct BreakpointManager {
    breakpoints: HashSet<u16>,
}

impl BreakpointManager {
    pub fn add_breakpoint(&mut self, address: u16) {
        self.breakpoints.insert(address);
    }

    pub fn remove_breakpoint(&mut self, address: u16) {
        self.breakpoints.remove(&address);
    }

    pub fn has_breakpoint(&self, address: u16) -> bool {
        self.breakpoints.contains(&address)
    }

    pub fn len(&self) -> usize {
        self.breakpoints.len()
    }

    pub fn clear(&mut self) {
        self.breakpoints.clear();
    }
}
