use std::ops::{Neg, Sub};
use std::time::SystemTime;

use chrono::Duration;

use crate::walk::traits::DirEntryWrapperExt;
use crate::GenericError;

pub trait Evaluate<E: DirEntryWrapperExt> {
    fn evaluate(&self, entry: &E) -> Result<bool, GenericError>;
}

pub trait DurationOffsetExt<T> {
    fn add_to(&self, absolute_time: T) -> T;
}

impl DurationOffsetExt<SystemTime> for Duration {
    fn add_to(&self, absolute_time: SystemTime) -> SystemTime {
        if self.num_milliseconds() < 0 {
            absolute_time.sub(self.neg().to_std().unwrap())
        } else {
            absolute_time.sub(self.to_std().unwrap())
        }
    }
}
