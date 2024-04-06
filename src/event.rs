use crate::percpu::this_cpu_data;

pub const IPI_EVENT_WAKEUP: usize = 0;
pub const IPI_EVENT_SHUTDOWN: usize = 1;

pub fn check_events() -> bool {
    let cpu_data = this_cpu_data();
    let event = {
        let _lock = cpu_data.ctrl_lock.lock();
        cpu_data.pending_event.take()
    };
    match event {
        Some(IPI_EVENT_WAKEUP) => {
            cpu_data.arch_cpu.run();
        }
        Some(IPI_EVENT_SHUTDOWN) => {
            cpu_data.arch_cpu.idle();
        }
        _ => false
    }
}