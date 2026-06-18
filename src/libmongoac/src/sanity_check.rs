/// Returns the input value unchanged.
#[unsafe(no_mangle)]
pub extern "C" fn mongoac_sanity_check(n: i32) -> i32 {
    n
}
