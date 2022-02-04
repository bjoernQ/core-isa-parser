use std::{fs, path::PathBuf, str::FromStr};

use anyhow::Result;
use enum_as_inner::EnumAsInner;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alpha1, alphanumeric1, char, digit1, hex_digit1, multispace1},
    combinator::{map_res, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded},
    IResult,
};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

fn main() -> Result<()> {
    for chip in Chip::iter() {
        let path = chip.core_isa_path()?;
        let text = fs::read_to_string(path)?;

        let parsed = text.parse::<DefinedValue>();
    }

    Ok(())
}

// ---------------------------------------------------------------------------

// Note that for the ESP32, since we are not using an RTOS we need to use the
// 'xtensa_esp108' overlay instead of the 'xtensa_esp32' overlay.
// https://docs.espressif.com/projects/esp-idf/en/v3.3.5/api-guides/jtag-debugging/tips-and-quirks.html
#[derive(Debug, Clone, Copy, PartialEq, Display, EnumIter)]
enum Chip {
    #[strum(to_string = "xtensa_esp108")]
    Esp32,
    #[strum(to_string = "xtensa_esp32s2")]
    Esp32s2,
    #[strum(to_string = "xtensa_esp32s3")]
    Esp32s3,
    #[strum(to_string = "xtensa_lx106")]
    Esp8266,
}

impl Chip {
    fn core_isa_path(&self) -> Result<PathBuf> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("xtensa-overlays")
            .join(self.to_string())
            .join("newlib/newlib/libc/sys/xtensa/include/xtensa/config/core-isa.h")
            .canonicalize()?;

        Ok(path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, EnumString)]
enum InterruptType {
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

#[derive(Debug, Clone, PartialEq, EnumAsInner)]
enum DefinedValue {
    Integer(i64),
    Interrupt(InterruptType),
    String(String),
}

impl FromStr for DefinedValue {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

// ---------------------------------------------------------------------------

fn comment(i: &str) -> IResult<&str, &str> {
    delimited(tag("/*"), is_not("*/"), tag("*/"))(i)
}

fn whitespace(i: &str) -> IResult<&str, Vec<&str>> {
    many0(alt((comment, multispace1)))(i)
}

fn decimal(i: &str) -> IResult<&str, i64> {
    map_res(digit1, |out: &str| i64::from_str_radix(&out, 10))(i)
}

fn hexadecimal(i: &str) -> IResult<&str, i64> {
    map_res(
        preceded(alt((tag("0x"), tag("0X"))), hex_digit1),
        |out: &str| i64::from_str_radix(&out, 16),
    )(i)
}

fn integer(i: &str) -> IResult<&str, i64> {
    alt((hexadecimal, decimal))(i)
}

fn string(i: &str) -> IResult<&str, &str> {
    delimited(char('"'), is_not("\""), char('"'))(i)
}

fn identifier(i: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))(i)
}

fn definition(i: &str) -> IResult<&str, &str> {
    todo!()
}
