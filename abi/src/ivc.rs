// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.

//! IVC information exported to a zone.

use crate::config::{HvIvcConfig, CONFIG_MAX_IVC_CONFIGS};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct IvcInfo {
    pub len: u64,
    pub ivc_ct_ipas: [u64; CONFIG_MAX_IVC_CONFIGS],
    pub ivc_shmem_ipas: [u64; CONFIG_MAX_IVC_CONFIGS],
    pub ivc_ids: [u32; CONFIG_MAX_IVC_CONFIGS],
    pub ivc_irqs: [u32; CONFIG_MAX_IVC_CONFIGS],
}

impl From<&[HvIvcConfig]> for IvcInfo {
    fn from(configs: &[HvIvcConfig]) -> Self {
        let mut ivc_ids = [0; CONFIG_MAX_IVC_CONFIGS];
        let mut ivc_ct_ipas = [0; CONFIG_MAX_IVC_CONFIGS];
        let mut ivc_shmem_ipas = [0; CONFIG_MAX_IVC_CONFIGS];
        let mut ivc_irqs = [0; CONFIG_MAX_IVC_CONFIGS];
        for (i, config) in configs.iter().enumerate() {
            if i >= CONFIG_MAX_IVC_CONFIGS {
                break;
            }
            ivc_ids[i] = config.ivc_id;
            ivc_ct_ipas[i] = config.control_table_ipa;
            ivc_shmem_ipas[i] = config.shared_mem_ipa;
            ivc_irqs[i] = config.interrupt_num;
        }
        Self {
            len: configs.len().min(CONFIG_MAX_IVC_CONFIGS) as u64,
            ivc_ids,
            ivc_shmem_ipas,
            ivc_ct_ipas,
            ivc_irqs,
        }
    }
}
