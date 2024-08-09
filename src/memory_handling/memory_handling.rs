use crate::config;

#[derive(Debug)]
pub struct MemoryHandler {
    current_memory: usize,
    memory_size_limit: usize,
}

impl MemoryHandler {
    pub fn new() -> Self {
        let limit_memory_value_for_initialize = config::Settings::new();
        MemoryHandler {
            current_memory: 0,
            memory_size_limit: megabytes_to_bytes(limit_memory_value_for_initialize.memory_size_limit)
        }
    }

    pub fn add_memory(&mut self, additional_memory: usize) {
        self.current_memory += additional_memory;
    }

    pub fn delete_memory(&mut self, memory_to_free: usize) {
        self.current_memory = self.current_memory.saturating_sub(memory_to_free);
    }

    pub fn is_memory_limit_finished(&self) -> bool {
        return self.current_memory > self.memory_size_limit ;
    }
}

fn megabytes_to_bytes(mb: usize) -> usize {
    mb * 1_048_576
}