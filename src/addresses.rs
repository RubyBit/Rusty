// Addresses namespace
pub mod func_defs {
    use std::os::raw::c_char;
    use libc::{uintptr_t};

    pub type RbxGetschedulerT = extern "cdecl" fn() -> uintptr_t;
    pub type RbxOutputT = extern "fastcall" fn(output_type: i16, str: *const c_char) -> (); // see if this works with normal char
    pub type RbxGetStateT = extern "thiscall" fn(SC: uintptr_t, state_type: *const i32) -> uintptr_t; //needs thiscall (need to test if i can use stdcall)
    pub type RbxPushvfstringT = extern "cdecl" fn(rl: uintptr_t, fmt: *const char, ...) -> i32;
    pub type RbxPseudo2adrT = extern "fastcall" fn(rl: uintptr_t, idx: i32) -> *const uintptr_t;
}

pub mod addresses {
    // These are offsets from base module (of the process)
    use libc::uintptr_t;

    pub const RBX_GETSCHEDULER_ADDY: uintptr_t = 0xDE05E0;
    pub const RBX_OUTPUT_ADDY: uintptr_t = 0x2eff60;
    pub const RBX_GETSTATE_ADDY: uintptr_t = 0x46B0B0;
    pub const RBX_PUSHVFSTRING_ADDY: uintptr_t = 0x1421b40;

    pub const SPAWN_FUNC_ADDY: uintptr_t = 0x4773A0;
    pub const DESERIALIZER_FUNC_ADDY: uintptr_t = 0x14A8C70; // luau_load

    pub const PUSHCCLOSURE_ADDY: uintptr_t = 0x12B3750;
    pub const PUSHCCLOSURE_EXIT_ADDY: uintptr_t = 0x12B39A9;

    pub const SETGLOBAL_ADDY: uintptr_t = 0x12B3E30;
    pub const SETGLOBAL_EXIT_ADDY: uintptr_t = 0x12B3FA2;
    pub const SETGLOBAL_PATH_1_ADDY: uintptr_t = 0x014A0C78;
    pub const SETGLOBAL_PATCH_2_ADDY: uintptr_t = 0x014A1010;

    pub const PSEUDO2ADR_ADDY: uintptr_t = 0x1421D03;
    pub const FAKE_RET_ADDY: uintptr_t = 0x1365AD1;

    pub const CALLCHECK_ADDY_DATA: uintptr_t = 0x3996ED4;
    pub const CALLCHECK_ADDY_CODE: uintptr_t = 0x2D42A7;
    pub const CALLCHECK_ADDY_VM: uintptr_t = 0x014F4AE0;

    pub const XOR_CONST: uintptr_t = 0x3718790;

    pub const PATCH_SPOT: uintptr_t = 0x0045AF72;
}

pub mod offsets {

    pub mod scheduler {
        use libc::uintptr_t;

        pub(crate) const JOBS_START: uintptr_t = 0x134;
        pub(crate) const JOBS_END: uintptr_t = 0x138;
        pub(crate) const FPS: uintptr_t = 0x118;
    }

    pub mod job {
        use libc::uintptr_t;

        pub(crate) const NAME: uintptr_t = 0x10;
    }

    pub mod waiting_scripts_job {
        use libc::uintptr_t;

        pub(crate) const DATAMODEL: uintptr_t = 0x28;
        pub(crate) const SCRIPT_CONTEXT: uintptr_t = 0x130;
    }

    pub mod identity {
        use libc::uintptr_t;

        pub const EXTRA_SPACE: uintptr_t = 0x48;
        pub const IDENTITY: uintptr_t = 0x18;
    }

    pub mod luastate {
        use libc::uintptr_t;

        pub const TOP: uintptr_t = 0xC;
        pub const BASE: uintptr_t = 0x10;
    }

    pub mod luafunc {
        use libc::uintptr_t;

        pub const FUNC: uintptr_t = 16;
    }
}
