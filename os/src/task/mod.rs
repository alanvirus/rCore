mod context;
mod manager;
mod pid;
mod processor;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::fs::OpenFlags;
use crate::{fs::open_file, loader::get_app_data_by_name};
use crate::sbi::shutdown;
use alloc::sync::Arc;
use lazy_static::*;
pub use manager::{fetch_task, TaskManager};
use task::{TaskControlBlock, TaskStatus};
use switch::__switch;

pub use context::TaskContext;
pub use manager::add_task;
pub use pid::{pid_alloc, KernelStack, PidAllocator, PidHandle};
pub use processor::{
    current_task, current_trap_cx, current_user_token, run_tasks, schedule, take_current_task,change_program_brk,
    Processor,
};

pub fn suspend_current_and_run_next() {
    let task = take_current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    let task_cx_ptr= &mut inner.task_cx as *mut TaskContext;
    inner.task_status = TaskStatus::Ready;
    drop(inner);
    add_task(task);
    schedule(task_cx_ptr);
}
pub const IDLE_PID: usize = 0;//* 
pub fn exit_current_and_run_next(exit_code: i32) {
    let task = take_current_task().unwrap();
    let pid = task.get_pid();
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        if exit_code != 0 {
            shutdown(true)
        } else {
            shutdown(false)
        }
    }
    let mut inner = task.inner_exclusive_access();
    inner.task_status = TaskStatus::Zombie;
    inner.exit_code =exit_code;
    {
        let mut initproc_inner =INITPROC.inner_exclusive_access();
        for child in inner.children.iter(){
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    inner.children.clear();
    inner.memory_set.recycle_data_pages();//为了销毁程序执行recycle_data_pages和两个drop
    drop(inner);
    drop(task);//这里如果task rc==0 会销毁KERNELSPACE中当前程序的内核栈，那么接下来会崩溃
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _)
}

lazy_static!{
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
        let inode=open_file("initproc", OpenFlags::RDONLY).unwrap();
        let v=inode.read_all();
        TaskControlBlock::new(v.as_slice())
    });
}
pub fn add_initproc(){
    add_task(INITPROC.clone());//INITPROC有两个Arc指向，一个是INITPROC，另一个在Manager中
}