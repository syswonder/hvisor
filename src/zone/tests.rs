use super::*;
use alloc::sync::Arc;
use spin::RwLock;

#[test_case]
fn test_add_and_remove_zone() {
    let zone_count = 50;
    let zone_count_before = ZONE_LIST.read().len();
    for i in 0..zone_count {
        let u8name_array = [i as u8; CONFIG_NAME_MAXLEN];
        let zone = Zone::new(i, &u8name_array);
        ZONE_LIST.write().push(Arc::new(RwLock::new(zone)));
    }
    for i in 0..zone_count {
        remove_zone(i);
    }
    assert_eq!(ZONE_LIST.read().len(), zone_count_before);
}
