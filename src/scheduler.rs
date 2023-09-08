
use std::ffi::{CStr};
use std::num::Wrapping;
use std::ptr::{copy_nonoverlapping};
use libc::{c_char, c_void, uintptr_t};

use crate::addresses::offsets;
use crate::{API};

pub struct Scheduler {
    pub(crate) task_scheduler: uintptr_t,
    pub(crate) datamodel: uintptr_t,
    pub(crate) script_context: uintptr_t
}

impl Scheduler {
    pub unsafe fn new() -> Scheduler {
        let task_scheduler = (API.get_scheduler)();

        let waiting_scripts_job = Scheduler::get_waiting_scripts_job(task_scheduler);
        Scheduler {
            task_scheduler,
            datamodel: *((waiting_scripts_job + offsets::waiting_scripts_job::DATAMODEL) as *const uintptr_t),
            script_context: *((waiting_scripts_job + offsets::waiting_scripts_job::SCRIPT_CONTEXT) as *const uintptr_t)
        }
    }

    pub fn get_global_luastate(&self) -> uintptr_t { // need to see if &Api is correct
        /*unsafe {
            let lua_state = Wrapping(*((self.script_context + 0x14C) as *const uintptr_t)) - Wrapping(self.script_context + 0x14C);
            return lua_state.0;
            //return ((lua_state + Wrapping(0x1C)) - Wrapping(*((lua_state.0 + 0x1C) as *const uintptr_t))).0;
        }
         */
        let state = 0;
        return (API.get_state)(self.script_context, &state as *const i32);
    }

    pub unsafe fn set_fps(&self, fps : f64) {
        *((self.task_scheduler + offsets::scheduler::FPS) as *mut f64) = (1 as f64) / fps;
    }

    unsafe fn get_waiting_scripts_job(task_scheduler: uintptr_t) -> uintptr_t {
        let mut last_job: uintptr_t = 0;
        //println!("In func (get_waiting_scripts_job)");
        for job in Scheduler::get_jobs(task_scheduler) {
            let job_name = CStr::from_ptr((job + offsets::job::NAME) as *const c_char).to_str();

            match job_name {
                Ok(..) => {}
                Err(..) => { // Bigger than 16 chars ( as such job_name tried to derive a string from a pointer)
                    let string = CStr::from_ptr(*((job + offsets::job::NAME) as *const uintptr_t) as *const c_char).to_str().unwrap();
                    // WaitingHybridScriptsJob bigger than 16 chars
                    if string == "WaitingHybridScriptsJob" {
                        //println!("POTENTIAL: {:#01x}", *((job + offsets::waiting_scripts_job::DATAMODEL) as *const uintptr_t));
                        last_job = job;
                    }
                }
            }
        }
        //println!("(get_waiting_scripts_job) I return -> {}", last_job);
        return last_job;
    }

    unsafe fn get_jobs(task_scheduler: uintptr_t) -> Vec<uintptr_t> {
        let mut jobs = Vec::new(); // or vec![]
        //println!("(get_jobs) In func");
        let mut current_job = *((task_scheduler + offsets::scheduler::JOBS_START) as *mut *mut uintptr_t);
        //println!("First job: {:#01x}", current_job as usize);
        loop {
            jobs.push(*current_job);
            current_job = current_job.add(2);
            if current_job == *((task_scheduler + offsets::scheduler::JOBS_END) as *mut *mut uintptr_t) {
                //println!("Last job: {:#01x}", current_job as usize);
                break;
            }
        }
        //println!("(get_jobs) I return");
        return jobs;
    }

    pub(crate) unsafe fn hook_waiting_scripts_job(&self,
                                                  hook: *mut c_void,
                                                  original_func: &mut uintptr_t,
                                                  task_scheduler: uintptr_t) {
        //println!("Hooking WaitingScriptsJob");
        // will need to check if works correctly
        let waiting_scripts_job = Scheduler::get_waiting_scripts_job(task_scheduler);
        //println!("Got waiting_scripts_job");

        let table= Box::into_raw(Box::new([0 as *mut c_void; 6]));
        let table_ptr: *mut c_void = table.cast();
        // Original slice
        //println!("Getting original slice {}", (*table)[2] as usize);
        copy_nonoverlapping(*(waiting_scripts_job as *const *const c_void), table_ptr, 6 * std::mem::size_of::<*mut c_void>());
        //println!("Copied waiting_scripts_job");

        //println!("Original func -> {}, SLICE -> {}", *original_func, (*table)[2] as usize);
        *original_func = (*table)[2] as uintptr_t;
        //println!("Placing hook -> {}", hook as usize);
        //slice.as_mut_ptr().offset(2).write(hook);
        (*table)[2] = hook;
        //println!("New slice placed.");
        *(waiting_scripts_job as *mut *mut [*mut c_void; 6]) = table; // [*mut c_void]
        println!("[Rusty] Hooked");

    }
}