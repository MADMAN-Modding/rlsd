pub mod conversions {
    /// This mod is to be used when dividing from bytes to another unit such as mebibytes or gibibytes
    pub mod byte {
        /// B -> KiB (1024^1) 
        pub const KIBIBYTE: f64 = 1024.0;
        /// B -> MiB (1024^2) 
        pub const MEBIBYTE: f64 = 1024.0 * 1024.0;
        /// B -> GiB (1024^3)
        pub const GIBIBYTE: f64 = 1024.0 * 1024.0 * 1024.0;
        /// B -> TiB (1024^4)
        pub const TEBIBYTE: f64 = 1024.0 * 1024.0 * 1024.0 * 1024.0;
    }
}