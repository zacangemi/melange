/// Tech → Dune term mapping
#[allow(dead_code)]
pub fn term<'a>(tech_name: &'a str) -> &'a str {
    match tech_name {
        "available_ram" => "Spice Reserves",
        "compatibility" => "Melange Yield",
        "gpu_cores" => "Maker Hooks",
        "context_window" => "Prescient Horizon",
        "tok_s" => "Spice Flow Rate",
        "swap_danger" => "The Desert Claims",
        "safe_to_run" => "The Spice Flows",
        "max_safe_context" => "Horizon Limit",
        "os_reserved" => "Spacing Guild Tax",
        "weight_memory" => "Spice Weight",
        "kv_cache" => "Prescient Cache",
        "overhead" => "Guild Overhead",
        "total_memory" => "Total Spice Demand",
        _ => tech_name,
    }
}
