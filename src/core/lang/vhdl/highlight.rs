use std::str::FromStr;

use colored::ColoredString;
use palette::*;
use colored::Colorize;

use crate::util::anyerror::AnyError;

pub type Rgb = (u8, u8, u8);

pub fn color(s: &str, hue: (u8, u8, u8)) -> ColoredString {
    s.truecolor(hue.0, hue.1, hue.2)
}

mod palette {
    use super::*;

    // reds
    pub const BURNT_ORANGE:     Rgb = (204,  85,   0);
    pub const GOLDEN_ROD:       Rgb = (0xDA, 0xA5, 0x20);
    pub const TOMATO:           Rgb = (0xFF, 0x63, 0x47);
    pub const ORANGE:           Rgb = (0xFF, 0xA5, 0x00);
    pub const RED:              Rgb = (0xFF, 0x00, 0x00);
    pub const FIREBRICK:        Rgb = (0xB2, 0x22, 0x22);

    //blues
    pub const DARK_CYAN:        Rgb = (0x00, 0x8B, 0x8B);
    pub const LT_SKY_BLUE:      Rgb = (135, 206, 250);
    pub const MED_BLUE:         Rgb = (0x00, 0x00, 0xCD);
    pub const TURQUOISE:        Rgb = (0x40, 0xE0, 0xD0);

    // greens
    pub const MED_SPRING_GREEN: Rgb = (  0, 250, 154);
    pub const SEAFOAM_GREEN:    Rgb = (159, 226, 191);
    pub const GREEN:            Rgb = (0x00, 0x80, 0x00);
    pub const LIME_GREEN:       Rgb = (0x32, 0xCD, 0x32);
    pub const PALM_LEAF:        Rgb = (0x64, 0x97, 0x50);
    pub const NATURE_GREEN:     Rgb = (0x4F, 0xAD, 0x27);
}


/* standard colorings */
pub const NUMBERS:                  Rgb = GOLDEN_ROD;
pub const CHARS:                    Rgb = SEAFOAM_GREEN;
pub const STRINGS:                  Rgb = BURNT_ORANGE;

/* `orbit get` colorings */ 
pub const SIGNAL_DEC_IDENTIFIER:    Rgb = LT_SKY_BLUE;
pub const INSTANCE_LHS_IDENTIFIER:  Rgb = LT_SKY_BLUE;
pub const DATA_TYPE:                Rgb = NATURE_GREEN;
pub const ENTITY_NAME:              Rgb = NATURE_GREEN;

#[derive(Debug, PartialEq)]
pub enum ColorMode {
    Always,
    Auto,
    Never,
}

impl FromStr for ColorMode {
    type Err = AnyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            _ => Err(AnyError(format!("value must be 'auto', 'always', or 'never'")))
        }
    }
}