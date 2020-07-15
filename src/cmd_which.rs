use crate::services::{StringOutputer, TaxfilePathGetter};

pub fn cmd_which(
    outputer: &mut dyn StringOutputer,
    taxfile_path_getter: &dyn TaxfilePathGetter,
) -> Result<(), String> {
    outputer.info(taxfile_path_getter.get_taxfile_path()?);
    Ok(())
}
