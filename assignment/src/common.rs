use std::{
    fmt,
    fmt::Display,
    fmt::Formatter,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use eyre::eyre;
use itertools::Itertools;

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<T>(pub Vec<Vec<T>>);

impl<T> Matrix<T> {
    pub fn shape(&self) -> (usize, usize) {
        (self.0.len(), self.0[0].len())
    }

    pub fn map<F, U>(&self, f: F) -> Matrix<U>
    where
        F: Fn(&T) -> U,
    {
        Matrix::<U>(
            self.iter()
                .map(|v| v.iter().map(&f).collect_vec())
                .collect_vec(),
        )
    }
}

impl<T> Deref for Matrix<T> {
    type Target = Vec<Vec<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Matrix<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> FromStr for Matrix<T>
where
    T: FromStr,
{
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split('\n')
            .filter_map(|s| {
                match s
                    .split_whitespace()
                    .map(|s| s.parse::<T>())
                    .try_collect::<_, Vec<T>, _>()
                {
                    Ok(v) if v.is_empty() => None,
                    x => Some(x),
                }
            })
            .try_collect()
            .ok()
            .and_then(|v: Vec<Vec<T>>| {
                if v.iter().map(|v| v.len()).all_equal() {
                    Some(Self(v))
                } else {
                    None
                }
            })
            .ok_or(eyre!("failed to parse Matrix"))
    }
}

impl<T> Display for Matrix<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !self.0.is_empty() {
            let m: Vec<Vec<String>> = self
                .0
                .iter()
                .map(|v| v.iter().map(|n| n.to_string()).collect())
                .collect();
            let result = m.iter().map(|v| v.join(" & ")).join(r" \\[1ex] ");
            write!(f, "{result}")?;
        }
        Ok(())
    }
}
