#![allow(clippy::missing_safety_doc)]

use std::io::Write;
use std::os::raw::{c_char, c_int, c_long, c_void};

#[repr(C)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct lua_CompileOptions {
    optimizationLevel: c_int,
    debugLevel: c_int,
    typeInfoLevel: c_int,
    coverageLevel: c_int,
    vectorLib: *const c_char,
    vectorCtor: *const c_char,
    vectorType: *const c_char,
    mutableGlobals: *const *const c_char,
    userdataTypes: *const *const c_char,
    librariesWithKnownMembers: *const *const c_char,
    libraryMemberTypeCb: Option<unsafe extern "C" fn(*const c_char, *const c_char) -> c_int>,
    libraryMemberConstantCb:
        Option<unsafe extern "C" fn(*const c_char, *const c_char, *mut *mut c_void)>,
    disabledBuiltins: *const *const c_char,
}

extern "C" {
    pub fn free(ptr: *mut c_void);

    pub fn luaL_newstate() -> *mut c_void;
    pub fn lua_close(state: *mut c_void);
    pub fn luaL_openlibs(state: *mut c_void);
    //pub fn lutec_opencrypto(state: *mut c_void);
    pub fn lutec_openfs(state: *mut c_void);
    pub fn lutec_openluau(state: *mut c_void);
    //pub fn lutec_opennet(state: *mut c_void);
    pub fn lutec_openprocess(state: *mut c_void);
    pub fn lutec_opentask(state: *mut c_void);
    pub fn lutec_openvm(state: *mut c_void);
    pub fn lutec_opensystem(state: *mut c_void);
    pub fn lutec_opentime(state: *mut c_void) -> c_int;
    pub fn lutec_setup_runtime(state: *mut c_void);
    pub fn lutec_destroy_runtime(state: *mut c_void) -> c_int;
    pub fn lua_gettop(state: *mut c_void) -> c_int;
    pub fn lua_settop(state: *mut c_void, index: c_int);
    pub fn lua_type(state: *mut c_void, index: c_int) -> c_int;
    pub fn lua_typename(state: *mut c_void, index: c_int) -> *const c_char;
    pub fn lua_remove(state: *mut c_void, index: c_int);
    pub fn lua_getfield(state: *mut c_void, index: c_int, k: *const c_char) -> c_int;
    pub fn lua_setfield(state: *mut c_void, index: c_int, k: *const c_char);
    pub fn lua_tolstring(state: *mut c_void, index: c_int, len: *mut c_long) -> *const c_char;
    pub fn lua_call(state: *mut c_void, nargs: c_int, nresults: c_int);
    pub fn lua_pcall(state: *mut c_void, nargs: c_int, nresults: c_int, errfunc: c_int) -> c_int;
    pub fn lua_newthread(state: *mut c_void) -> *mut c_void;
    pub fn lua_pushvalue(state: *mut c_void, index: c_int);
    pub fn lua_resume(state: *mut c_void, from: *mut c_void, narg: c_int) -> c_int;
    pub fn lua_isnumber(state: *mut c_void, index: c_int) -> c_int;
    pub fn lua_isthread(state: *mut c_void, index: c_int) -> c_int;
    pub fn lua_yield(state: *mut c_void, from: *mut c_void, narg: c_int) -> c_int;
    pub fn lua_xmove(state: *mut c_void, from: *mut c_void, n: c_int);
    pub fn lua_xpush(state: *mut c_void, from: *mut c_void, n: c_int);
    pub fn lua_resetthread(state: *mut c_void) -> c_int;
    pub fn luaL_errorL(state: *mut c_void, format: *const c_char, ...) -> !;

    pub fn lua_pushinteger(state: *mut c_void, n: c_int);
    pub fn lua_tointegerx(state: *mut c_void, index: c_int, isnum: *mut c_int) -> c_int;
    pub fn lua_pushcclosurek(
        L: *mut c_void,
        f: unsafe extern "C-unwind" fn(L: *mut c_void) -> c_int,
        debugname: *const c_char,
        nup: c_int,
        cont: *const c_void,
    );

    pub fn lua_createtable(state: *mut c_void, narr: c_int, nrec: c_int);
    pub fn lua_setmetatable(state: *mut c_void, index: c_int) -> c_int;
    pub fn lua_getmetatable(state: *mut c_void, index: c_int) -> c_int;
    pub fn lua_getmetatablepointer(state: *mut c_void, index: c_int) -> *const c_void;
    pub fn lua_topointer(state: *mut c_void, index: c_int) -> *const c_void;

    pub fn luau_compile(
        source: *const c_char,
        size: usize,
        options: *mut lua_CompileOptions,
        outsize: *mut usize,
    ) -> *mut c_char;
    pub fn luau_load(
        state: *mut c_void,
        chunkname: *const c_char,
        data: *const c_char,
        size: usize,
        env: c_int,
    ) -> c_int;

    pub fn lutec_set_runtimeinitter(callback: lutec_setupState_init) -> c_int;
    pub fn lua_checkstack(state: *mut c_void, extra: c_int) -> c_int;
    pub fn lua_tothread(state: *mut c_void, idx: c_int) -> *mut c_void;

}

/*
extern "C" const int LUTE_STATE_MISSING_ERROR = 0;
extern "C" const int LUTE_STATE_ERROR = 1;
extern "C" const int LUTE_STATE_SUCCESS = 2;
extern "C" const int LUTE_STATE_EMPTY = 3;
extern "C" const int LUTE_STATE_UNSUPPORTED_OP = 4;

extern "C" struct RunOnceResult
{
    int op = LUTE_STATE_UNSUPPORTED_OP; // Default to unsupported operation
    lua_State *state = nullptr;         // The lua_State that was run, if applicable
};
*/

pub const LUTE_STATE_MISSING_ERROR: c_int = 0;
pub const LUTE_STATE_ERROR: c_int = 1;
pub const LUTE_STATE_SUCCESS: c_int = 2;
pub const LUTE_STATE_EMPTY: c_int = 3;
pub const LUTE_STATE_UNSUPPORTED_OP: c_int = 4;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct RunOnceResult {
    pub op: c_int,          // Operation result code
    pub state: *mut c_void, // The lua_State that was run, if applicable
}

extern "C-unwind" {
    pub fn lutec_run_once(state: *mut c_void) -> RunOnceResult;
    pub fn lutec_run_once_lua(state: *mut c_void) -> c_int;
    pub fn lutec_has_work(state: *mut c_void) -> c_int;
    pub fn lutec_has_threads(state: *mut c_void) -> c_int;
    pub fn lutec_has_continuation(state: *mut c_void) -> c_int;
}

/*
typedef struct
{
    lua_State *L;
} lua_State_wrapper;
*/

#[repr(C)]
pub struct lua_State_wrapper {
    pub parent: *mut c_void,
    pub L: *mut c_void,
    pub DC: *mut c_void, // Pointer to the data copy VM, if applicable
    pub runtime_to_set: *mut c_void, // Pointer to the runtime to set with lua_setthreaddata in the wrapper
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct lutec_setupState {
    pub setup_lua_state: unsafe extern "C-unwind" fn(wrapper: *mut lua_State_wrapper),
}

// Populates function pointers in the given lutec_setupState.
pub type lutec_setupState_init = unsafe extern "C" fn(config: *mut lutec_setupState);

#[cfg(not(target_os = "emscripten"))]
extern "C" {
    pub fn luau_codegen_supported() -> c_int;
    pub fn luau_codegen_create(state: *mut c_void);
    pub fn luau_codegen_compile(state: *mut c_void, idx: c_int);
}

pub unsafe fn lua_getglobal(state: *mut c_void, k: *const c_char) {
    lua_getfield(state, -1002002 /* LUA_GLOBALSINDEX */, k);
}

pub unsafe fn lua_setglobal(state: *mut c_void, k: *const c_char) {
    lua_setfield(state, -1002002 /* LUA_GLOBALSINDEX */, k);
}

pub unsafe fn to_string<'a>(state: *mut c_void, index: c_int) -> &'a str {
    let mut len: c_long = 0;
    let ptr = lua_tolstring(state, index, &mut len);

    if ptr.is_null() {
        println!("Error: lua_tolstring returned null");
        return "";
    }
    if len < 0 {
        println!("Error: length is negative");
        return "";
    }

    let bytes = std::slice::from_raw_parts(ptr as *const u8, len as usize);
    std::str::from_utf8(bytes).unwrap()
}

pub unsafe fn set_lute_state_initter() -> c_int {
    pub unsafe extern "C" fn init_config(config: *mut lutec_setupState) {
        unsafe extern "C-unwind" fn setup_lua_state(wrapper: *mut lua_State_wrapper) {
            let state = luaL_newstate();
            if state.is_null() {
                return;
            }
            luaL_openlibs(state);
            (*wrapper).L = state;
        }

        (*config).setup_lua_state = setup_lua_state;
    }

    lutec_set_runtimeinitter(init_config)
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use super::*;

    #[test]
    fn test_luau() {
        println!("Running Luau tests...");
        unsafe {
            let state = luaL_newstate();
            assert!(!state.is_null());

            // Enable JIT if supported
            #[cfg(not(target_os = "emscripten"))]
            if luau_codegen_supported() != 0 {
                luau_codegen_create(state);
            }

            luaL_openlibs(state);

            lua_getglobal(state, c"_VERSION".as_ptr());
            let version = to_string(state, -1);

            assert_eq!(version, "Luau");

            let code = "local a, b = ... return a + b";
            let mut bytecode_size = 0;
            let bytecode = luau_compile(
                code.as_ptr().cast(),
                code.len(),
                ptr::null_mut(),
                &mut bytecode_size,
            );
            let result = luau_load(state, c"sum".as_ptr(), bytecode, bytecode_size, 0);
            assert_eq!(result, 0);
            free(bytecode.cast());

            // Compile the function (JIT, if supported)
            #[cfg(not(target_os = "emscripten"))]
            if luau_codegen_supported() != 0 {
                luau_codegen_compile(state, -1);
            }

            // Call the loaded function
            lua_pushinteger(state, 123);
            lua_pushinteger(state, 321);
            lua_call(state, 2, 1);
            assert_eq!(lua_tointegerx(state, -1, ptr::null_mut()), 444);

            lua_close(state);
        }
    }

    #[test]
    fn test_lute_open() {
        println!("Running Lute tests...");
        unsafe {
            println!("initter result: {}", set_lute_state_initter());

            let state = luaL_newstate();
            lutec_setup_runtime(state);
            assert!(!state.is_null());
            println!("state: {:?}", state);
            println!("gettop: {}", lua_gettop(state));

            // Enable JIT if supported
            #[cfg(not(target_os = "emscripten"))]
            if luau_codegen_supported() != 0 {
                luau_codegen_create(state);
            }

            luaL_openlibs(state);

            /*
                {"@lute/crypto", luteopen_crypto},
                {"@lute/fs", luteopen_fs},
                {"@lute/luau", luteopen_luau},
                {"@lute/net", luteopen_net},
                {"@lute/process", luteopen_process},
                {"@lute/task", luteopen_task},
                {"@lute/vm", luteopen_vm},
                {"@lute/system", luteopen_system},
                {"@lute/time", luteopen_time},
            */
            //lutec_opencrypto(state);
            //lua_setglobal(state, c"crypto".as_ptr());

            lutec_openfs(state);
            lua_setglobal(state, c"fs".as_ptr());

            lutec_openluau(state);
            lua_setglobal(state, c"luau".as_ptr());

            //lutec_opennet(state);
            //lua_setglobal(state, c"net".as_ptr());

            lutec_openprocess(state);
            lua_setglobal(state, c"process".as_ptr());

            lutec_opentask(state);
            lua_setglobal(state, c"task".as_ptr());

            lutec_openvm(state);
            lua_setglobal(state, c"vm".as_ptr());

            lutec_opensystem(state);
            lua_setglobal(state, c"system".as_ptr());

            lutec_opentime(state);
            lua_setglobal(state, c"time".as_ptr());

            lua_getglobal(state, c"_VERSION".as_ptr());
            let version = to_string(state, -1);

            assert_eq!(version, "Luau");

            let code = "return (tostring(time.duration.seconds(2) + time.duration.seconds(3)))";
            let mut bytecode_size = 0;
            let bytecode = luau_compile(
                code.as_ptr().cast(),
                code.len(),
                ptr::null_mut(),
                &mut bytecode_size,
            );
            let result = luau_load(state, c"sum".as_ptr(), bytecode, bytecode_size, 0);
            assert_eq!(result, 0);
            free(bytecode.cast());

            // Compile the function (JIT, if supported)
            #[cfg(not(target_os = "emscripten"))]
            if luau_codegen_supported() != 0 {
                luau_codegen_compile(state, -1);
            }

            // Call the loaded function
            lua_pushinteger(state, 123);
            lua_pushinteger(state, 321);

            if lua_pcall(state, 2, 1, 0) != 0 {
                println!("error running function `f-a': {}", to_string(state, -1));
            }

            assert_eq!(lua_tointegerx(state, -1, ptr::null_mut()), 5);

            println!("gettop call one: {}", lua_gettop(state));

            // Remove the result from the stack
            while lua_gettop(state) > 0 {
                // lua_settop(L, -(n)-1)
                lua_settop(state, -2);
            }

            println!("gettop call two: {}", lua_gettop(state));

            lutec_destroy_runtime(state);
            lua_close(state);
        }
    }

    #[test]
    fn test_metatablepointer() {
        unsafe {
            let state = luaL_newstate();
            assert!(!state.is_null());

            lua_createtable(state, 0, 0);
            assert!(lua_getmetatablepointer(state, -1).is_null());

            lua_createtable(state, 0, 0);
            let mt_ptr1 = lua_topointer(state, -1);

            lua_setmetatable(state, -2);
            let mt_ptr2 = lua_getmetatablepointer(state, -1);
            assert_eq!(mt_ptr1, mt_ptr2);

            lua_close(state);
        }
    }

    #[test]
    fn test_exceptions() {
        unsafe {
            let state = luaL_newstate();
            assert!(!state.is_null());

            unsafe extern "C-unwind" fn it_panics(state: *mut c_void) -> c_int {
                luaL_errorL(state, "exception!\0".as_ptr().cast());
            }

            lua_pushcclosurek(state, it_panics, ptr::null(), 0, ptr::null());
            let result = lua_pcall(state, 0, 0, 0);
            assert_eq!(result, 2); // LUA_ERRRUN
            assert_eq!(to_string(state, -1), "exception!");
        }
    }
}
