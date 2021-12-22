use std::{collections::HashMap, env, fs, path::PathBuf, str::FromStr};

use anyhow::Result;
use regex::Regex;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumIter)]
enum Chip {
    #[strum(to_string = "esp32")]
    Esp32,
    #[strum(to_string = "esp32s2")]
    Esp32s2,
    #[strum(to_string = "esp32s3")]
    Esp32s3,
    #[strum(to_string = "esp8266")]
    Esp8266,
}

impl Chip {
    fn core_isa_path(&self) -> Result<PathBuf> {
        use Chip::*;

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("xtensa-overlays")
            .join(match self {
                Esp32 => "xtensa_esp32",
                Esp32s2 => "xtensa_esp32s2",
                Esp32s3 => "xtensa_esp32s3",
                Esp8266 => "xtensa_lx106",
            })
            .join(PathBuf::from(
                "newlib/newlib/libc/sys/xtensa/include/xtensa/config/core-isa.h",
            ))
            .canonicalize()?;

        Ok(path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, EnumString)]
enum IntType {
    #[strum(serialize = "XTHAL_INTTYPE_EXTERN_EDGE")]
    ExternEdge,
    #[strum(serialize = "XTHAL_INTTYPE_EXTERN_LEVEL")]
    ExternLevel,
    #[strum(serialize = "XTHAL_INTTYPE_NMI")]
    Nmi,
    #[strum(serialize = "XTHAL_INTTYPE_PROFILING")]
    Profiling,
    #[strum(serialize = "XTHAL_INTTYPE_SOFTWARE")]
    Software,
    #[strum(serialize = "XTHAL_INTTYPE_TIMER")]
    Timer,
    #[strum(serialize = "XTHAL_TIMER_UNCONFIGURED")]
    TimerUnconfigured,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Value {
    Integer(i64),
    Interrupt(IntType),
}

fn main() -> Result<()> {
    for chip in Chip::iter() {
        let defines = find_all_defines(chip)?;
        let _config = parse_defines(defines)?;
    }

    Ok(())
}

fn find_all_defines(chip: Chip) -> Result<Vec<String>> {
    let path = chip.core_isa_path()?;
    let lines = fs::read_to_string(path)?
        .lines()
        .filter_map(|line| {
            if line.starts_with("#define") {
                Some(line.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(lines)
}

fn parse_defines(defines: Vec<String>) -> Result<HashMap<String, Value>> {
    let re_define = Regex::new(r"^#define[\s]+([a-zA-Z\d_]+)[\s]+([^\s]+)")?;
    let re_ident = Regex::new(r"^[a-zA-Z\d_]+$")?;

    // Iterate through each line containing a definition. Attempt to match the
    // various components and map identifiers to values. In the case that a value is
    // another identifier, keep track of this for the next pass.
    let mut map = HashMap::new();
    let mut dependants = HashMap::new();

    let mut unmatched = 0;
    for define in defines {
        if !re_define.is_match(&define) {
            unmatched += 1;
            continue;
        }

        let captures = re_define.captures(&define).unwrap();
        let identifier = captures.get(1).unwrap().as_str().to_string();
        let value = captures.get(2).unwrap().as_str().to_string();

        if let Ok(integer) = value.parse::<i64>() {
            // Decimal integer literal
            map.insert(identifier, Value::Integer(integer));
        } else if let Ok(integer) = i64::from_str_radix(&value.replace("0x", ""), 16) {
            // Hexadecimal integer literal
            map.insert(identifier, Value::Integer(integer));
        } else if let Ok(interrupt) = IntType::from_str(&value) {
            // Interrupt type
            map.insert(identifier, Value::Interrupt(interrupt));
        } else if re_ident.is_match(&value) {
            // Identifier
            dependants.insert(identifier, value);
        } else {
            println!("Unrecognized value: {}", value);
        }
    }

    println!("{} defines matched, {} unmatched", map.len(), unmatched);

    // Once we have iterated through all of the definitions, we can address any
    // dependant identifiers.
    for (identifier, value) in dependants {
        if map.contains_key(&value) {
            map.insert(identifier.clone(), *map.get(&value).unwrap());
        } else {
            println!("Dependant not found: {} = {}", identifier, value);
        }
    }

    Ok(map)
}
