use crate::percpu::this_cpu_data;

pub const IPI_EVENT_WAKEUP: usize = 0;
pub const IPI_EVENT_SHUTDOWN: usize = 1;

pub fn check_events() -> bool {
    let cpu_data = this_cpu_data();
    let mut _lock = Some(cpu_data.ctrl_lock.lock());
    match cpu_data.pending_event {
        Some(IPI_EVENT_WAKEUP) => {
            cpu_data.arch_cpu.run();
        }
        Some(IPI_EVENT_SHUTDOWN) => {
            todo!();
        }
        _ => false
    }
}