#![feature(abi_thiscall)]
#![feature(naked_functions)]

#[macro_use]
extern crate lazy_static;

extern crate core;

use std::{thread};
use std::ffi::{CString};
use std::thread::sleep;
use std::time::Duration;
use crate::api::{Api, Roblox};
use crate::console::Console;
use crate::execution::Execution;
use crate::scheduler::Scheduler;

mod addresses;
mod api;
mod console;
mod execution;
mod scheduler;
mod lua_opcodes;

// x86
// rustup target add i686-pc-windows-msvc
// cargo build --target=i686-pc-windows-msvc
// rustup target remove i686-pc-windows-msvc

lazy_static! {  // Initialize at runtime, uses the base address of the Roblox process
    static ref ROBLOX: Roblox = Roblox::new();
    static ref API: Api = Api::new(&ROBLOX);
    static ref SCHEDULER: Scheduler = unsafe { Scheduler::new() };
}


unsafe fn main_fn() {
    let _console = Console::new(CString::new("Rusty").expect("CString::new failed").as_ptr());
    (API.output)(0, CString::new("Initializing rusty..").expect("CSTRING FAILED").as_ptr());
    println!("No console Rusty!");
    println!("[Rusty] Task scheduler -> {:#01x}", SCHEDULER.task_scheduler);
    println!("[Rusty] Datamodel -> {:#01x}", SCHEDULER.datamodel);
    println!("[Rusty] Script Context -> {:#01x}", SCHEDULER.script_context);
    println!("[Rusty] Lua state -> {:#01x}", SCHEDULER.get_global_luastate());


    let mut execution = Execution::new(&SCHEDULER);
    println!("[Rusty] EXECUTION STRUCT ID -> {:p}", &execution);
    println!("[Rusty] Hooking..");

    // hook and run script
    sleep(Duration::from_millis(1000));
    execution.hook_waiting_scripts_job();

    println!("[Rusty] Setting identity to 7..");
    execution.set_identity(7);

    // Executing script
    sleep(Duration::from_millis(1000));

    let path = std::path::Path::new(r"D:\yield.txt");
    let contents = std::fs::read_to_string(path).expect("Unable to read file");
    execution.run_script(contents.as_str());



    //let lua = Lua::new();
    //let chunk = lua.load(&script).exec().expect("Failed to run");
    //println!("Lua script number -> {}", lua.globals().get::<_, u32>("num").expect("failed to get var"))

}

#[no_mangle]
pub extern "stdcall" fn DllMain(
    hinst_dll: winapi::shared::minwindef::HINSTANCE,
    fwd_reason: winapi::shared::minwindef::DWORD,
    lpl_reserved: winapi::shared::minwindef::LPVOID
) -> i32 {
    if fwd_reason == winapi::um::winnt::DLL_PROCESS_ATTACH {
        let thread = thread::spawn(move || {
            unsafe {
                main_fn()};
        });
        drop(thread);
    }
    return 1 as i32;
}
