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
}

impl Unit {
    /// Converts the unit to a usize
    ///
    /// # Returns
    /// * `usize` - Value of the unit in Bytes
    pub fn to_uint(&self) -> usize {
        match self {
            Self::BYTE => byte_to_unit::BYTE,
            Self::KIBIBYTE => byte_to_unit::KIBIBYTE,
            Self::MEBIBYTE => byte_to_unit::MEBIBYTE,
            Self::GIBIBYTE => byte_to_unit::GIBIBYTE,
            Self::TEBIBYTE => byte_to_unit::TEBIBYTE,
            Self::PEBIBYTE => byte_to_unit::PEBIBYTE,
            Self::Percentage => 100,
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
        }
    }
}

/// This module is to be used when dividing from bytes to another unit such as mebibytes or gibibytes
pub mod byte_to_unit {
    /// B -> B (2^0)
    pub const BYTE: usize = usize::pow(2, 0);
    /// B -> KiB (2^10)
    pub const KIBIBYTE: usize = usize::pow(2, 10);
    /// B -> MiB (2^20)
    pub const MEBIBYTE: usize = usize::pow(2, 20);
    /// B -> GiB (2^30)
    pub const GIBIBYTE: usize = usize::pow(2, 30);
    /// B -> TiB (2^40)
    pub const TEBIBYTE: usize = usize::pow(2, 40);
    /// B -> PiB (2^50)
    pub const PEBIBYTE: usize = usize::pow(2, 50);
}

/// Formats bytes to the smallest possible unit
pub fn format_bytes(bytes: f64, unit: Unit) -> f64 {
    if bytes >= 1024.0 {
        format_bytes(bytes / 1024.0, unit.next())
    } else {
        bytes
    }
}

pub fn get_unit(bytes: usize, unit: Unit) -> Unit {
    if bytes >= 1024 {
        get_unit(bytes / 1024, unit.next())
    } else {
        unit.clone()
    }
}