
use std::ffi::CString;
use terminal_spinners::{SpinnerBuilder, DOTS};
use std::num::Wrapping;
use std::sync::Mutex;
use libc::{c_void, uintptr_t};
use mlua::{Compiler};
use crate::{API, Roblox, ROBLOX, Scheduler, SCHEDULER};
use crate::lua_opcodes::{get_opcode_from_byte, LuauOpcode};

static mut SCRIPT_QUEUE: Vec<Vec<u8>> = vec![];
static mut ORIGINAL_FUNC: uintptr_t = 0;
static MUTEX: Mutex<i32> = Mutex::new(0);

pub struct Execution<'a> {
    pub(crate) scheduler: &'a Scheduler,
}

impl Execution<'_> {
    pub(crate) fn new(scheduler: &Scheduler) -> Execution {
        Execution {scheduler}
    }


    extern "fastcall" fn scheduler_cycle(waiting_scripts_job: uintptr_t, _fake_arg: i32, a2: i32) -> i32 {
        let lock = MUTEX.lock().unwrap();
        let r1 = SCHEDULER.get_global_luastate();
        unsafe {
            if !SCRIPT_QUEUE.is_empty() {

                println!("[Rusty] Scheduler has been stepped");
                //println!("Got bytecode -> {}", SCRIPT_QUEUE.get(0).expect("Script_queue invalid").get(0).expect("Bytecode invalid"));
                let mut bytecode = SCRIPT_QUEUE.pop().unwrap();

                drop(lock);

                if bytecode.get(0).unwrap().eq(&(0 as u8)) {
                    //println!("Bytecode is problematic");
                    bytecode.remove(0);
                    // error
                    let error_header = CString::new("RUSTY ERROR -> ").unwrap();
                    let error_msg = CString::from_vec_with_nul(bytecode).unwrap();

                    let error = error_header.to_str().unwrap().to_owned() + error_msg.to_str().unwrap();
                    (API.output)(1, CString::new(error).expect("CString error").as_ptr());
                } else {
                    println!("[Rusty] Script bytecode valid to execute");

                    /*for byte in bytecode.iter() {
                        println!("{}", byte);
                    }

                     */

                    // execute script
                    let bytecode_len = bytecode.len(); //need to do before value is moved
                    // translate u8 vector to C String
                    let bytecode_string = CString::from_vec_unchecked(bytecode); // is this safe?

                    let chunk_name = CString::new("¬Rusty¬").expect("CString error");

                    //println!("Bytecode converted to CString");
                    // byte len is done with the bytes (can be done with the string if necessary)

                    ROBLOX.deserialize(r1,chunk_name.as_ptr(), bytecode_string.as_ptr(), (bytecode_len) as i32);
                    println!("Deserialized");

                    ROBLOX.other_spawn(r1);

                    println!("Spawned script");

                    Roblox::decrement_top(r1, 1);
                    //println!("Decremented top");
                }
            }
        }
        // call function to execute script
        unsafe {
            let function: extern "thiscall" fn(job: uintptr_t, a2: i32) -> i32 = std::mem::transmute(ORIGINAL_FUNC as *const ());
            function(waiting_scripts_job, a2)
        }
    }

    pub(crate) fn hook_waiting_scripts_job(&mut self) {
        let hooked_func = Self::scheduler_cycle;
        unsafe {
            self.scheduler.hook_waiting_scripts_job(hooked_func as *mut c_void, &mut ORIGINAL_FUNC, self.scheduler.task_scheduler);
            println!("[Rusty] Original function -> {}", ORIGINAL_FUNC);
        }
    }

    pub(crate) fn run_script(&mut self, script: &str) {
        // initialize spinner
        let spinner = SpinnerBuilder::new().spinner(&DOTS).text("Compiling script...").start();

        let compiler = Compiler::new();
        let mut compiled_script = compiler.compile(script);
        // get bytecode specification (need to encode opcodes)
        // constant index
        let mut offset = 1;
        // get number of constants
        let num_of_constants = Execution::read_var_int(&compiled_script, &mut offset);
        //println!("Number of constants -> {}", num_of_constants);
        // go through constants
        for _ in 1..=num_of_constants {
            let constant_length = Execution::read_var_int(&compiled_script, &mut offset);
            //println!("Constant length -> {}", constant_length);
            offset += constant_length as usize;
        }
        // go through functions (the proto table)
        let num_of_functions = Execution::read_var_int(&compiled_script, &mut offset);
        //println!("Number of functions -> {}", num_of_functions);
        for _ in 1..=num_of_functions {
            offset += 4;
            let num_of_opcodes = Execution::read_var_int(&compiled_script, &mut offset);
            //println!("Number of opcodes -> {}", num_of_opcodes);
            let mut instruction_counter = offset;
            let mut byte_counter = 0;
            let mut double_inc = false;
            let counter = offset;
            loop {
                if instruction_counter == counter + num_of_opcodes as usize {
                    break;
                }

                if byte_counter == 0 {

                    if double_inc {
                        //println!("Going through double inc..");
                        instruction_counter += 1;
                        double_inc = false;
                        byte_counter = 3;
                        offset += 1;
                        continue;
                    }

                    let byte = compiled_script.get_mut(offset).unwrap();
                    //println!("OPCODE -> {}", *byte);
                    instruction_counter += 1;

                    if Execution::get_op_length(get_opcode_from_byte(*byte).unwrap()) == 2 {
                        //println!("DOUBLE INSTRUCTION");
                        byte_counter = 4; // need to check this out
                        double_inc = true;

                        *byte = (Wrapping(*byte) * Wrapping(227)).0; // Encoding opcode
                        //println!("NEW OPCODE -> {}", *byte);
                        continue;
                    }
                    else {
                        byte_counter = 4;
                    }

                    *byte = (Wrapping(*byte) * Wrapping(227)).0;
                    //println!("NEW OPCODE -> {}", *byte);
                }

                byte_counter -= 1;
                offset += 1;
            }
            offset += 3;
            // needs more function analysis (to reach appropriate offset for next function)
            let size_k = Execution::read_var_int(&compiled_script, &mut offset);
            //println!("SIZE_K -> {}", size_k);

            for _ in 0..size_k {
                let lbc = compiled_script.get(offset).unwrap();
                offset += 1;
                match *lbc {
                    1=> offset += 1, // LBC_CONSTANT_BOOLEAN
                    2=> offset += 8, // LBC_CONSTANT_NUMBER
                    3=> {Execution::read_var_int(&compiled_script, &mut offset); }, // LBC_CONSTANT_STRING
                    4=> offset += 4, // LBC_CONSTANT_IMPORT
                    5=> { // LBC_CONSTANT_TABLE
                        let keys = Execution::read_var_int(&compiled_script, &mut offset);
                        for _ in 0..keys {
                            Execution::read_var_int(&compiled_script, &mut offset); // key
                        }
                    },
                    6=> {Execution::read_var_int(&compiled_script, &mut offset);}, // LBC_CONSTANT_CLOSURE
                    _=> {}
                }
            }


            let size_p = Execution::read_var_int(&compiled_script, &mut offset);
            //println!("SIZE_P -> {}", size_p);
            for _ in 0..size_p {
                Execution::read_var_int(&compiled_script, &mut offset); // proto
            }
            // maybe just offset += 1
            Execution::read_var_int(&compiled_script, &mut offset); // Line defined
            Execution::read_var_int(&compiled_script, &mut offset); // Debug name

            let line_info = compiled_script.get(offset).unwrap();
            offset += 1;

            //println!("LINE_INFO -> {}", line_info);
            if *line_info == 1 {
                let line_ga_plog2 = compiled_script.get(offset).unwrap();
                //println!("LINE_GA_PLOG2 -> {}", line_ga_plog2);
                offset += 1; // linegaplog2
                for _ in 0..num_of_opcodes {
                    offset += 1; // last offset
                }

                let intervals = ((num_of_opcodes - 1) >> *line_ga_plog2 as u32) + (1);
                //println!("INTERVALS -> {}", intervals);
                for _ in 0..intervals {
                    offset += 4; // last line
                }
            }

            let debug_info = compiled_script.get(offset).unwrap();
            offset += 1;
            //println!("DEBUG_INFO -> {}", debug_info);
            if *debug_info == 1 {
                let size_loc_vars = Execution::read_var_int(&compiled_script, &mut offset);
                for _ in 0..size_loc_vars {
                    Execution::read_var_int(&compiled_script, &mut offset); // var name
                    Execution::read_var_int(&compiled_script, &mut offset); // start pc
                    Execution::read_var_int(&compiled_script, &mut offset); // end pc
                    Execution::read_var_int(&compiled_script, &mut offset); // reg
                }

                let size_upvalues = Execution::read_var_int(&compiled_script, &mut offset);
                for _ in 0..size_upvalues {
                    Execution::read_var_int(&compiled_script, &mut offset); // upvalue name
                }
            }
        }
        // push to script queue
        let _guard = MUTEX.lock().unwrap();
        unsafe {SCRIPT_QUEUE.push(compiled_script); }
        spinner.text("Script compiled.");
        spinner.done();
    }

    pub fn set_identity(&self, identity: i8) {
        Roblox::set_identity(self.scheduler.get_global_luastate(), identity);
    }

    fn get_op_length(op: LuauOpcode) -> i32 {
        match op {
            LuauOpcode::LopGetglobal | LuauOpcode::LopSetglobal
            | LuauOpcode::LopGetimport | LuauOpcode::LopGettableks
            | LuauOpcode::LopSettableks | LuauOpcode::LopNamecall
            | LuauOpcode::LopJumpifeq | LuauOpcode::LopJumpifle
            | LuauOpcode::LopJumpiflt | LuauOpcode::LopJumpifnoteq
            | LuauOpcode::LopJumpifnotle | LuauOpcode::LopJumpifnotlt
            | LuauOpcode::LopNewtable | LuauOpcode::LopSetlist
            | LuauOpcode::LopForgloop | LuauOpcode::LopLoadkx
            | LuauOpcode::LopJumpifeqk | LuauOpcode::LopJumpifnoteqk
            | LuauOpcode::LopFastcall2 | LuauOpcode::LopFastcall2k => 2,
            _=>1
        }
    }

    fn read_var_int(data: &Vec<u8>, offset: &mut usize) -> u32 {
        let mut result: u32 = 0;
        let mut shift = 0;

        let mut byte;

        loop {
            byte = *(*data).get(*offset).unwrap();
            result |= ((byte as u32 & 127) << shift) as u32;
            shift += 7;
            if (byte & 128) != 128 {
                break;
            }
            *offset += 1;
        }
        *offset += 1;
        result
    }

}
