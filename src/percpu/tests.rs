use super::*;
use alloc::vec::Vec;

#[test_case]
fn test_cpuset() {
    let mut cpuset = CpuSet::new(3, 0b1010);
    assert_eq!(cpuset.contains_cpu(0), false);
    assert_eq!(cpuset.contains_cpu(1), true);
    assert_eq!(cpuset.contains_cpu(2), false);
    assert_eq!(cpuset.contains_cpu(3), true);
    cpuset.set_bit(0);
    assert_eq!(cpuset.contains_cpu(0), true);
    assert_eq!(cpuset.contains_cpu(1), true);
    assert_eq!(cpuset.contains_cpu(2), false);
    assert_eq!(cpuset.contains_cpu(3), true);
    cpuset.clear_bit(1);
    assert_eq!(cpuset.contains_cpu(0), true);
    assert_eq!(cpuset.contains_cpu(1), false);
    assert_eq!(cpuset.contains_cpu(2), false);
    assert_eq!(cpuset.contains_cpu(3), true);
    assert_eq!(cpuset.first_cpu(), Some(0));
    assert_eq!(cpuset.iter().collect::<Vec<_>>(), vec![0, 3]);
    assert_eq!(cpuset.iter_except(0).collect::<Vec<_>>(), vec![3]);
}
