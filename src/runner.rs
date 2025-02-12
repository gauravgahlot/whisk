use super::*;

fn run_module(module: &[u8]) -> Option<i32> {
    let sections = super::sections::parse_sections(module);

    // parse type and function sections
    let type_section = parse_type_section(&sections.types); // Vec<FuncSignature>
    let func_types = parse_function_section(&sections.functions); // Vec<u32>

    // locate the `_start` function in the exports
    let start_func_idx = sections
        .exports
        .iter()
        .find(|export| export.name == "_start" && export.kind == ExportKind::Function)
        .and_then(|export| export.index)
        .expect("No _start function found");

    // access the corresponding function body and its type index
    let func = &sections.funcs[start_func_idx];
    let type_index = func_types[start_func_idx];

    // create a context and registry
    let mut ctx = Context::new();
    let registry = ImportRegistry::new();

    // execute `_start` function
    execute_function(&mut ctx, func, &registry, &type_section, type_index)
}
