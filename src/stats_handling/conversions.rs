/// This module is to be used when dividing from bytes to another unit such as mebibytes or gibibytes
pub mod byte_to_unit {
    /// B -> B (2^0)
    pub const BYTE: u128 = u128::pow(2, 0);
    /// B -> KiB (2^10)
    pub const KIBIBYTE: u128 = u128::pow(2, 10);
    /// B -> MiB (2^20)
    pub const MEBIBYTE: u128 = u128::pow(2, 20);
    /// B -> GiB (2^30)
    pub const GIBIBYTE: u128 = u128::pow(2, 30);
    /// B -> TiB (2^40)
    pub const TEBIBYTE: u128 = u128::pow(2, 40);
    /// B -> PiB (2^50)
    pub const PEBIBYTE: u128 = u128::pow(2, 50);
}

pub mod time_to_second {
    pub const SECOND: u128 = 1;
    pub const MINUTE: u128 = 60;
    pub const HOUR: u128 = 60 * MINUTE;
    pub const DAY: u128 = 24 * HOUR;
    pub const WEEK: u128 = 7 * DAY;
    pub const YEAR: u128 = 52 * WEEK;
    pub const DECADE: u128 = 10 * YEAR;
}

#[derive(Clone)]
pub enum Unit {
    /// B (1024^0)
    BYTE,
    /// KiB (1024^1)
    KIBIBYTE,
    /// MiB (1024^2)
    MEBIBYTE,
    /// GiB (1024^3)
    GIBIBYTE,
    /// TiB (1024^4)
    TEBIBYTE,
    /// PiB (1024^5)
    PEBIBYTE,
    Percentage,
    /// s (60^1)
    SECOND,
    /// min (60^1)
    MINUTE,
    /// hour (60^2)
    HOUR,
    /// day (hour * 24)
    DAY,
    /// week (day * 7)
    WEEK,
    /// year (week * 52)
    YEAR,
    /// decade (week * 52)
    DECADE
}

impl Unit {
    /// Converts the unit to a usize
    ///
    /// # Returns
    /// * `usize` - Value of the unit in Bytes
    pub fn to_uint(&self) -> u128 {
        match self {
            Self::BYTE => byte_to_unit::BYTE,
            Self::KIBIBYTE => byte_to_unit::KIBIBYTE,
            Self::MEBIBYTE => byte_to_unit::MEBIBYTE,
            Self::GIBIBYTE => byte_to_unit::GIBIBYTE,
            Self::TEBIBYTE => byte_to_unit::TEBIBYTE,
            Self::PEBIBYTE => byte_to_unit::PEBIBYTE,
            Self::Percentage => 100,
            Self::SECOND => time_to_second::SECOND,
            Self::MINUTE => time_to_second::MINUTE,
            Self::HOUR => time_to_second::HOUR,
            Self::DAY => time_to_second::DAY,
            Self::WEEK => time_to_second::WEEK,
            Self::YEAR => time_to_second::YEAR,
            Self::DECADE => time_to_second::DECADE
        }
    }

    pub fn to_f64(&self) -> f64 {
        self.to_uint() as f64
    }

    /// Returns the next highest Unit
    pub fn next(&self) -> Unit {
        match self {
            Self::BYTE => Self::KIBIBYTE,
            Self::KIBIBYTE => Self::MEBIBYTE,
            Self::MEBIBYTE => Self::GIBIBYTE,
            Self::GIBIBYTE => Self::TEBIBYTE,
            Self::TEBIBYTE => Self::PEBIBYTE,
            Self::PEBIBYTE => Self::PEBIBYTE,
            Self::Percentage => Self::Percentage,
            Self::SECOND => Self::MINUTE,
            Self::MINUTE => Self::HOUR,
            Self::HOUR => Self::DAY,
            Self::DAY => Self::WEEK,
            Self::WEEK => Self::YEAR,
            Self::YEAR => Self::DECADE,
            Self::DECADE => Self::DECADE
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::BYTE => "B",
            Self::KIBIBYTE => "KiB",
            Self::MEBIBYTE => "MiB",
            Self::GIBIBYTE => "GiB",
            Self::TEBIBYTE => "TiB",
            Self::PEBIBYTE => "PiB",
            Self::Percentage => "%",
            Self::SECOND => "s",
            Self::MINUTE => "minutes",
            Self::HOUR => "hours",
            Self::DAY => "days",
            Self::WEEK => "weeks",
            Self::YEAR => "years",
            Self::DECADE => "decades"
        }
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BYTE => write!(f, "{}", self.to_str()),
            Self::KIBIBYTE => write!(f, "{}", self.to_str()),
            Self::MEBIBYTE => write!(f, "{}", self.to_str()),
            Self::GIBIBYTE => write!(f, "{}", self.to_str()),
            Self::TEBIBYTE => write!(f, "{}", self.to_str()),
            Self::PEBIBYTE => write!(f, "{}", self.to_str()),
            Self::Percentage => write!(f, "{}", self.to_str()),
            Self::SECOND => write!(f, "{}", self.to_str()),
            Self::MINUTE => write!(f, "{}", self.to_str()),
            Self::HOUR => write!(f, "{}", self.to_str()),
            Self::DAY => write!(f, "{}", self.to_str()),
            Self::WEEK => write!(f, "{}", self.to_str()),
            Self::YEAR => write!(f, "{}", self.to_str()),
            Self::DECADE => write!(f, "{}", self.to_str())
        }
    }
}

/// Formats bytes to the smallest possible unit
/// 
/// # Arguments
/// * `bytes: f64` - The number of bytes to convert
/// * `unit: Unit` - The unit to start from
/// 
/// # Returns
/// `f64` - The number of bytes in the smallest possible unit
pub fn format_bytes(bytes: f64, unit: Unit) -> f64 {
    if bytes >= 1024.0 {
        format_bytes(bytes / 1024.0, unit.next())
    } else {
        bytes
    }
}

/// Returns the smallest unit for the given bytes
/// 
/// # Arguments
/// * `bytes: usize` - The number of bytes to convert
/// * `unit: Unit` - The unit to start from
/// 
/// # Returns
/// `Unit` - The smallest unit for the given bytes
pub fn get_byte_unit(bytes: usize, unit: Unit) -> Unit {
    if bytes >= 1024 {
        get_byte_unit(bytes / 1024, unit.next())
    } else {
        unit.clone()
    }
}

/// Formats time in seconds to the smallest possible unit
/// 
/// # Arguments
/// * `seconds: u64` - The number of seconds to convert
/// * `unit: Unit` - The unit to start from
/// 
/// # Returns
/// `f64` - The number of seconds in the smallest possible unit
pub fn format_time(seconds: u128, unit: Unit) -> f64 {
    let next_unit = unit.next();
    if seconds >= next_unit.to_uint() as u128 {
        format_time(seconds, next_unit)
    } else {
        seconds as f64 / unit.to_uint() as f64
    }
}

/// Returns the smallest unit for the given seconds
/// 
/// # Arguments
/// * `seconds: u64` - The number of seconds to convert
/// * `unit: Unit` - The unit to start from
/// 
/// # Returns
/// `Unit` - The smallest unit for the given seconds
pub fn get_time_unit(seconds: u128, unit: Unit) -> Unit {
    let next_unit = unit.next();
    if seconds >= next_unit.to_uint() as u128 {
        get_time_unit(seconds, next_unit)
    } else {
        unit.clone()
    }
}