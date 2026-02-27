/// Lookup table for Apple Silicon memory bandwidth (GB/s).
/// This is the critical factor for tok/s estimation on unified memory chips.
pub fn lookup(cpu_brand: &str) -> f64 {
    let brand = cpu_brand.to_lowercase();

    // M4 family
    if brand.contains("m4 ultra") { return 800.0; }
    if brand.contains("m4 max") { return 546.0; }
    if brand.contains("m4 pro") { return 273.0; }
    if brand.contains("m4") { return 120.0; }

    // M3 family
    if brand.contains("m3 ultra") { return 800.0; }
    if brand.contains("m3 max") { return 300.0; }
    if brand.contains("m3 pro") { return 150.0; }
    if brand.contains("m3") { return 100.0; }

    // M2 family
    if brand.contains("m2 ultra") { return 800.0; }
    if brand.contains("m2 max") { return 400.0; }
    if brand.contains("m2 pro") { return 200.0; }
    if brand.contains("m2") { return 100.0; }

    // M1 family
    if brand.contains("m1 ultra") { return 800.0; }
    if brand.contains("m1 max") { return 400.0; }
    if brand.contains("m1 pro") { return 200.0; }
    if brand.contains("m1") { return 68.0; }

    // Unknown — conservative estimate
    100.0
}
