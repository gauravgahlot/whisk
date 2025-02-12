#[no_mangle]
#[inline(always)]
pub extern "C" fn plus_one(left: i32) -> i32 {
    left + 1
}
