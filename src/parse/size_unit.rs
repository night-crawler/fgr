use crate::mk_filter_enum;

mk_filter_enum!(SizeUnit, SIZE_UNIT_ALIASES, [
    Byte: "B",
    Kilobyte: "Kb", "K",
    Megabyte: "Mb", "M",
    Gigabyte: "Gb", "G",
    Terabyte: "Tb", "T"
]);

impl SizeUnit {
    pub fn to_bytes(&self, value: usize) -> usize {
        match self {
            Self::Byte => value,
            Self::Kilobyte => value * 1000,
            Self::Megabyte => value * 1000 * 1000,
            Self::Gigabyte => value * 1000 * 1000 * 1000,
            Self::Terabyte => value * 1000 * 1000 * 1000 * 1000,
        }
    }
}
