pub fn get_current_space_id() -> u64 {
    unsafe {
        let conn = SLSMainConnectionID();
        CGSGetActiveSpace(conn)
    }
}

#[link(name = "SkyLight", kind = "framework")]
unsafe extern "C" {
    fn CGSGetActiveSpace(conn: u32) -> u64;
    fn SLSMainConnectionID() -> u32;
}
