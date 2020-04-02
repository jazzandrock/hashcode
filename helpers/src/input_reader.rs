use std::convert::AsRef;
use std::fmt::Debug;
use std::str::FromStr;

#[derive(Default)]
pub struct InputReader {
    s: String,
}

impl InputReader {
    pub fn new() -> Self {
        Self { s: String::new() }
    }

    pub fn ints_from_line<T: FromStr + Debug>(&mut self) -> Result<Vec<T>, <T as FromStr>::Err> {
        let cin = std::io::stdin();
        cin.read_line(&mut self.s).unwrap();

        let res = self
            .s
            .split_whitespace()
            .map(T::from_str)
            .collect::<Result<Vec<_>, _>>()?;

        self.s = String::new();

        Ok(res)
    }
}

pub trait SliceToTuple<U: Copy> {
    fn tuple_2(&self) -> (U, U);
    fn tuple_3(&self) -> (U, U, U);
    fn tuple_4(&self) -> (U, U, U, U);
}

impl<T, U> SliceToTuple<U> for T
where
    T: AsRef<[U]>,
    U: Copy,
{
    fn tuple_2(&self) -> (U, U) {
        match self.as_ref() {
            [a, b] => (*a, *b),
            _ => panic!("no 2 arguments"),
        }
    }
    fn tuple_3(&self) -> (U, U, U) {
        match self.as_ref() {
            [a, b, c] => (*a, *b, *c),
            _ => panic!("no 3 arguments"),
        }
    }
    fn tuple_4(&self) -> (U, U, U, U) {
        match self.as_ref() {
            [a, b, c, d] => (*a, *b, *c, *d),
            _ => panic!("no 4 arguments"),
        }
    }
}
