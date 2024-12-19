use super::*;

#[test_case]
fn test_simple_send_event() {
    init(1);
    send_event(0, 0, IPI_EVENT_WAKEUP);
    assert_eq!(fetch_event(0), Some(IPI_EVENT_WAKEUP));
}
