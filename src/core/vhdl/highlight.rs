use colored::ColoredString;
use palette::*;
use colored::Colorize;

pub type Rgb = (u8, u8, u8);

pub fn color(s: &str, hue: (u8, u8, u8)) -> ColoredString {
    s.truecolor(hue.0, hue.1, hue.2)
}

mod palette {
    use super::*;

    pub const BURNT_ORANGE: Rgb = (204, 85, 0);
    pub const MED_SPRING_GREEN: Rgb = (0, 250, 154);
    pub const LT_SKY_BLUE: Rgb = (135, 206, 250);
    pub const SEAFOAM_GREEN: Rgb = (159, 226, 191);
}

/* standard colorings */
pub const NUMBERS: Rgb = SEAFOAM_GREEN;
pub const CHARS: Rgb = BURNT_ORANGE;


/* `orbit get` colorings */
pub const SIGNAL_DEC_IDENTIFIER: Rgb = LT_SKY_BLUE;
pub const INSTANCE_LHS_IDENTIFIER: Rgb = LT_SKY_BLUE;
pub const DATA_TYPE: Rgb = SEAFOAM_GREEN;
pub const ENTITY_NAME: Rgb = SEAFOAM_GREEN;