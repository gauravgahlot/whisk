use std::collections::HashMap;

use super::executor;

pub struct ImportRegistry {
    functions: HashMap<String, fn(&mut executor::Context, &[i32]) -> Option<i32>>,
}

impl ImportRegistry {
    pub fn new() -> Self {
        ImportRegistry {
            functions: HashMap::new(),
        }
    }

    pub fn add_function(
        &mut self,
        module: &str,
        name: &str,
        func: fn(&mut executor::Context, &[i32]) -> Option<i32>,
    ) {
        let key = format!("{}.{}", module, name);
        self.functions.insert(key, func);
    }

    pub fn resolve_function(
        &self,
        module: &str,
        name: &str,
    ) -> Option<&fn(&mut executor::Context, &[i32]) -> Option<i32>> {
        let key = format!("{}.{}", module, name);
        self.functions.get(&key)
    }
}
