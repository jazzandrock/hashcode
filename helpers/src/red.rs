use std::char;
use std::fmt::Debug;
use std::iter::Iterator;
use std::str::FromStr;

fn read<T, B>(iter: &mut B) -> T
where
    T: FromStr + Debug + Default,
    B: Iterator<Item = u8>,
{
    #[derive(Copy, Clone)]
    enum State {
        SkipWhiteSpace,
        CopyCharacters,
    }

    let mut state = State::SkipWhiteSpace;

    let mut buf = [0 as u8; 20];
    let mut i = 0;
    for c in iter {
        match state {
            State::SkipWhiteSpace => {
                if !(c as char).is_whitespace() {
                    buf[i] = c as u8;
                    i += 1;

                    state = State::CopyCharacters;
                }
            }
            State::CopyCharacters => {
                if (c as char).is_whitespace() {
                    break;
                }

                buf[i] = c as u8;
                i += 1;
            }
        }
    }

    unsafe {
        let s = std::str::from_utf8_unchecked(&buf[0..i]);
        T::from_str(&s).ok().unwrap()
    }
}

pub struct Red<B: Iterator<Item = u8>> {
    iter: B,
}

impl<B: Iterator<Item = u8>> Red<B> {
    pub fn new(iter: B) -> Self {
        Self { iter }
    }

    pub fn read<T: FromStr + Debug + Default>(&mut self) -> T {
        read::<T, _>(&mut self.iter)
    }

    pub fn read_vec<T: FromStr + Debug + Default>(&mut self, n: usize) -> Vec<T> {
        let mut res = Vec::with_capacity(n);
        for _ in 0..n {
            res.push(self.read::<T>());
        }
        res
    }
}
