use std::os::raw::{c_char, c_int, c_long, c_void};

extern "C" {
    pub fn luaL_newstate() -> *mut c_void;
    pub fn luaL_openlibs(state: *mut c_void);
    pub fn lua_getfield(state: *mut c_void, index: c_int, k: *const c_char);
    pub fn lua_tolstring(state: *mut c_void, index: c_int, len: *mut c_long) -> *const c_char;
    pub fn luaL_loadstring(state: *mut c_void, s: *const c_char) -> c_int;
    pub fn lua_getglobal(state: *mut c_void, k: *const c_char);
}

#[test]
fn lua_works() {
    use std::slice;
    unsafe {
        let state = luaL_newstate();
        assert!(!state.is_null());

        luaL_openlibs(state);

        let version = {
            lua_getglobal(state, "_VERSION\0".as_ptr().cast());
            let mut len: c_long = 0;
            let version_ptr = lua_tolstring(state, -1, &mut len);
            slice::from_raw_parts(version_ptr as *const u8, len as usize)
        };

        assert_eq!(version, "Lua 5.2".as_bytes());
    }
}

#[test]
fn unicode_identifiers() {
    unsafe {
        let state = luaL_newstate();
        let code = "local 😀 = 0\0";
        let ret = luaL_loadstring(state, code.as_ptr().cast());
        assert_ne!(0, ret);
    }
}
