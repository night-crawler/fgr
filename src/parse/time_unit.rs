pub enum TimeUnit {
    Second(usize),
    Minute(usize),
    Hour(usize),
    Day(usize),
    Month(usize),
    Year(usize)
}


enum TimeUnitError {

}

impl TryFrom<(isize, &str)> for TimeUnit {
    type Error = ();

    fn try_from(value: (isize, &str)) -> Result<Self, Self::Error> {
        todo!()
    }
}