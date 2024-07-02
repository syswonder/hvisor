use crate::error::HvResult;

pub fn init_hv_page_table(fdt: &fdt::Fdt) -> HvResult {
    info!("loongarch64: mm: init_hv_page_table");
    Ok(())
}
