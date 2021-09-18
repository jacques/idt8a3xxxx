#![no_std]

//! idt8a3xxxx: A crate for drivers for the Renesas 8A3XXXX series
//!
//! The Renesas (nee IDT) 8A3XXXX series (branded as ClockMatrix) is a family
//! of clock generator parts.  These parts are sophisticated and capable,
//! offering a great degree of programmability.  This crate making available a
//! static definition of the familiy's modules and registers; the "8A3xxxx
//! Family Programming Guide" has details as to their meaning.  The
//! definitions themselves are contained in a RON file that, at build time
//! via `build.rs`, is turned into the static definition.
//!
#[derive(Debug, PartialEq)]
pub enum Contents {
    Byte,
    Word,
    Word24,
    Word32,
    Word40,
    Word48,
    Frequency,
    TimeOfDay,
}

///
/// Registers are selected by selecting the page that contains them in
/// PAGE_ADDR -- but in I2C 1B mode (the default), only PAGE_ADDR[15:8]
/// is relevant: PAGE_ADDR[16:31] is hardcoded, and PAGE_ADDR[7:0] is
/// set as part of the I2C transaction.
///
pub const PAGE_ADDR: u8 = 0xfd;

pub fn page(addr: u16) -> u8 {
    (addr >> 8) as u8
}

impl Contents {
    fn size(&self) -> u16 {
        match self {
            Contents::Byte => 1,
            Contents::Word => 2,
            Contents::Word24 => 3,
            Contents::Word32 => 4,
            Contents::Word40 => 5,
            Contents::Word48 => 6,
            Contents::Frequency => 8,
            Contents::TimeOfDay => 11,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Payload<'a> {
    contents: Contents,
    data: &'a [u8],
}

impl<'a> Payload<'a> {
    pub fn from_slice(contents: Contents, slice: &'a [u8]) -> Option<Self> {
        let len = contents.size() as usize;

        if slice.len() < len {
            None
        } else {
            Some(Self {
                contents: contents,
                data: &slice[0..len],
            })
        }
    }

    pub fn value(&self) -> u64 {
        let mut rval = 0u64;

        for i in 0..self.data.len() {
            rval |= (self.data[i] as u64) << (i * 8);
        }

        rval
    }
}

#[derive(Debug)]
pub struct Register<'a> {
    name: &'a str,
    offset: u16,
    contents: Contents,
}

#[derive(Debug)]
pub struct Module<'a> {
    name: &'a str,
    base: &'a [u16],
    registers: &'a [Register<'a>],
}

include!(concat!(env!("OUT_DIR"), "/modules.rs"));

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::*;

    #[test]
    fn dump() {
        let modules = modules();
        println!("{:#x?}", modules);
    }

    #[test]
    fn overlap() {
        let modules = modules();
        let mut seen = 0;

        for module in modules {
            for i in 0..module.base.len() {
                let base = module.base[i];

                let name = if module.base.len() > 1 {
                    format!("{}_{}", module.name, i)
                } else {
                    format!("{}", module.name)
                };

                for register in module.registers {
                    let addr = base + register.offset;
                    let limit = addr + register.contents.size();
                    assert!(addr >= seen);
                    println!(
                        "0x{:04x} - 0x{:04x}:  {}.{}",
                        addr,
                        limit - 1,
                        name,
                        register.name
                    );
                    seen = addr + register.contents.size();
                }
            }
        }
    }

    #[test]
    fn data() {
        let bytes = [0xde, 0x01, 0xce, 0xfa, 0xed, 0xfe];

        let check = [
            (Contents::Byte, 0xdeu64),
            (Contents::Word, 0x1de),
            (Contents::Word24, 0xce01de),
            (Contents::Word32, 0xface01de),
            (Contents::Word40, 0xedface01de),
            (Contents::Word48, 0xfeedface01de),
        ];

        for c in check {
            let p = Payload::from_slice(c.0, &bytes).unwrap();
            println!("{:x}", p.value());
            assert_eq!(p.value(), c.1);
        }
    }
}