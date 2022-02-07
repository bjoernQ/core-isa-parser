use strum::IntoEnumIterator;
use xtensa_core_isa::{get_config, Chip};
use anyhow::Result;

fn main() -> Result<()> {
    for chip in Chip::iter() {
        println!("{}", chip);
        let config = get_config(chip);
        println!("\n{:#x?}\n\n", config);
    }

    Ok(())
}
