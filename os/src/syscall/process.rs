use crate::fs::{open_file, OpenFlags};
use crate::mm::{translated_ref, translated_refmut, translated_str};
use crate::task::{
     current_process,  current_user_token, exit_current_and_run_next,
    pid2process, suspend_current_and_run_next, SignalAction, SignalFlags, MAX_SIG,
};
use crate::timer::get_time_ms;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_sbrk(size: i32) -> isize {
    // if let Some(old_brk) = change_program_brk(size) {
    //     old_brk as isize
    // } else {
    //     -1
    // }
    size;
    -1
}

pub fn sys_getpid() -> isize {
    current_process().get_pid() as isize
}

pub fn sys_fork() -> isize {
    let current_process = current_process();
    let new_process = current_process.fork();
    let new_pid = new_process.get_pid();
    let trap_cx = new_process.inner_exclusive_access().tasks[0]
        .as_ref()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx();
    trap_cx.x[10] = 0;
    new_pid as isize
}

pub fn sys_exec(path: *const u8, mut args: *const usize) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    let mut args_vec: Vec<String> = Vec::new();
    loop {
        let arg_str_ptr = *translated_ref(token, args);
        if arg_str_ptr == 0 {
            break;
        }
        args_vec.push(translated_str(token, arg_str_ptr as *const u8));
        unsafe {
            args = args.add(1);
        }
    }
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let process = current_process();
        let argc = args_vec.len();
        process.exec(all_data.as_slice(), args_vec);
        // return argc because cx.x[10] will be covered with it later
        argc as isize
    } else {
        -1
    }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    //if pid is a zombie, release it and write its exit_code in exit_code_ptr (virtaddr)
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.get_pid())
    {
        return -1;
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        p.inner_exclusive_access().is_zombie && (pid == -1 || pid as usize == p.get_pid())
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        assert_eq!(Arc::strong_count(&child), 1); //to be drop totally soon
        let found_pid = child.get_pid();
        let exit_code = child.inner_exclusive_access().exit_code;
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
}

pub fn sys_kill(pid: usize, signal: u32) -> isize {
    if let Some(process) = pid2process(pid) {
        if let Some(flag) = SignalFlags::from_bits(signal) {
            process.inner_exclusive_access().signals |= flag;
            0
        } else {
            -1
        }
    } else {
        -1
    }
}

pub fn sys_sigprocmask(mask: u32) -> isize {
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    let old_mask = inner.signal_mask;
    if let Some(flag) = SignalFlags::from_bits(mask) {
        inner.signal_mask = flag;
        old_mask.bits() as isize
    } else {
        -1
    }
}

pub fn sys_sigreturn() -> isize {
    let process = current_process();
        let mut inner = process.inner_exclusive_access();
        inner.handling_sig = -1;
        // restore the trap context
        let trap_ctx = inner.get_task(0).inner_exclusive_access().get_trap_cx();
        *trap_ctx = inner.trap_ctx_backup.unwrap();
        // Here we return the value of a0 in the trap_ctx,
        // otherwise it will be overwritten after we trap
        // back to the original execution of the application.
        trap_ctx.x[10] as isize
}

fn check_sigaction_error(signal: SignalFlags, action: usize, old_action: usize) -> bool {
    if action == 0
        || old_action == 0
        || signal == SignalFlags::SIGKILL
        || signal == SignalFlags::SIGSTOP
    {
        true
    } else {
        false
    }
}

pub fn sys_sigaction(
    signum: i32,
    action: *const SignalAction,
    old_action: *mut SignalAction,
) -> isize {
    let token = current_user_token();
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    if signum as usize > MAX_SIG {
        return -1;
    }
    if let Some(flag) = SignalFlags::from_bits(1 << signum) {
        if check_sigaction_error(flag, action as usize, old_action as usize) {
            return -1;
        }
        let prev_action = inner.signal_actions.table[signum as usize];
        *translated_refmut(token, old_action) = prev_action;
        inner.signal_actions.table[signum as usize] = *translated_ref(token, action);
        0
    } else {
        -1
    }
}
