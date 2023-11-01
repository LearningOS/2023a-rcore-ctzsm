//! Process management syscalls
use core::mem;
use core::slice;

use crate::config::PAGE_SIZE;
use crate::mm::translated_byte_buffer;
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, get_current_task_info,
        current_user_token, task_mmap, task_munmap,
    }, timer::{get_time_ms, get_time_us},
};

/// TimeVal for time methods
#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    /// Second.
    pub sec: usize,
    /// u second.
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    pub status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}
/* 
unsafe fn struct_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    slice::from_raw_parts((p as *const T) as * const u8, mem::size_of::<T>())
} */

fn copy_kernel_result_to_user_space<T: Sized>(ptr: *mut T, data: T) {
    let sz = mem::size_of::<T>();
    let slice = unsafe { slice::from_raw_parts((&data as *const T) as *const u8, sz) };
    let mut buffers = translated_byte_buffer(current_user_token(), ptr as *const u8, sz);
    let mut buffer_index: usize = 0;
    let mut buffer_len: usize = 0;
    for (i, b) in slice.iter().enumerate() {
        if buffers[buffer_index].len() <= i - buffer_len {
            buffer_len += buffers[buffer_index].len();
            buffer_index += 1;
        }
        buffers[buffer_index][i - buffer_len] = *b;
    }
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let res = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    copy_kernel_result_to_user_space(_ts, res);
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let res = get_current_task_info();
    let data = TaskInfo {
        status: res.status,
        syscall_times: res.syscall_times,
        time: get_time_ms() - res.time,
    };
    copy_kernel_result_to_user_space(_ti, data);
    0
}

/// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if _port & !0x7 != 0 || _port & 0x7 == 0 {
        return -1;
    }
    if _start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }
    task_mmap(_start, _len, _port)
}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if _start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }
    task_munmap(_start, _len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
