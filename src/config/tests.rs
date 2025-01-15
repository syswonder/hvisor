use super::*;

#[test_case]
fn test_simple_config() {
    #[cfg(target_arch = "aarch64")]
    {
        let config = HvZoneConfig::new(
            0,
            0b1111,
            1,
            [HvConfigMemoryRegion::new_empty(); CONFIG_MAX_MEMORY_REGIONS],
            1,
            [0; CONFIG_MAX_INTERRUPTS],
            1,
            [HvIvcConfig::default(); CONFIG_MAX_IVC_CONGIGS],
            0,
            0,
            0,
            0,
            0,
            [0; CONFIG_NAME_MAXLEN],
            HvArchZoneConfig {
                gicd_base: 0,
                gicr_base: 0,
                gicd_size: 0,
                gicr_size: 0,
                gits_base: 0,
                gits_size: 0,
            },
            HvPciConfig::new_empty(),
            0,
            [0; CONFIG_MAX_PCI_DEV],
        );
        assert_eq!(config.cpus(), vec![0, 1, 2, 3]);
        assert_eq!(config.memory_regions().len(), 1);
        assert_eq!(config.interrupts().len(), 1);
        assert_eq!(config.ivc_config().len(), 1);
    }
}
