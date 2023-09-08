//CONSOLE

use std::ffi::CString;
use libc::{c_char};
use winapi::um::consoleapi::AllocConsole;
use winapi::um::wincon::{SetConsoleTitleA};
use winapi::um::winnt::{LPSTR};
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};


pub struct Console;

impl Console {
    pub unsafe fn new(title: *const c_char) -> Console {

        let free_console_ptr = GetProcAddress(GetModuleHandleA(
            CString::new("kernel32.dll").expect("CString::new failed").as_ptr()),
                                              CString::new("FreeConsole").expect("CString::new failed").as_ptr());

        let _guard = region::protect_with_handle(free_console_ptr, 1, region::Protection::READ_WRITE_EXECUTE).unwrap();
        *(free_console_ptr as *mut i32) = 0xC3;

        AllocConsole();


        SetConsoleTitleA(title as LPSTR);

        return Console {
        }
    }
}