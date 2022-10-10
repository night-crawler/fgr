use std::num::TryFromIntError;

use strum_macros::EnumIter;

#[derive(Debug, Eq, PartialEq, EnumIter)]
pub enum SizeUnit {
    Byte(usize),
    Kb(usize),
    Mb(usize),
    Tb(usize),
}


impl SizeUnit {
    pub fn get_aliases(&self) -> &'static [&'static str] {
        match self {
            SizeUnit::Byte(_) => &[][..],
            SizeUnit::Kb(_) => &["Kb"][..],
            SizeUnit::Mb(_) => &["Mb"][..],
            SizeUnit::Tb(_) => &["Tv"][..],
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SizeUnitError {
    #[error("Cast Error: {0}")]
    CastError(#[from] TryFromIntError),

    #[error("Unknown size unit specifier: {0}")]
    UnknownSpecifierError(String),
}

impl TryFrom<(isize, &str)> for SizeUnit {
    type Error = SizeUnitError;

    fn try_from((size, unit): (isize, &str)) -> Result<Self, Self::Error> {
        let size: usize = size.try_into()?;

        match unit {
            "Mb" => Ok(SizeUnit::Mb(size)),
            "Kb" => Ok(SizeUnit::Kb(size)),
            "Tb" => Ok(SizeUnit::Tb(size)),
            "" => Ok(SizeUnit::Byte(size)),
            _ => Err(SizeUnitError::UnknownSpecifierError(unit.to_string())),
        }
    }
}
