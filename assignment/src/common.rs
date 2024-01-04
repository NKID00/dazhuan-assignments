use std::{
    fmt,
    fmt::Display,
    fmt::Formatter,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use eyre::eyre;
use itertools::Itertools;
use num::{BigRational, One, Signed};

pub use crate::linalg::ReducedRowEchelonForm;

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
        if !self.is_empty() {
            write!(
                f,
                "{}",
                self.iter()
                    .map(|v| v.iter().map(|n| n.to_string()).join(" & "))
                    .join(r" \\[1ex] ")
            )?;
        }
        Ok(())
    }
}

pub trait ToTex {
    fn to_tex(&self) -> String;
    fn to_tex_with_positive_sign(&self) -> String;
    fn to_tex_with_paren(&self) -> String;
    fn to_tex_ignore_one(&self) -> String;
    fn to_tex_with_sign_ignore_one(&self) -> String;
    fn sign_to_tex(&self) -> String;
    fn sign_to_tex_with_positive_sign(&self) -> String;
}

impl ToTex for BigRational {
    fn to_tex(&self) -> String {
        if self.is_integer() {
            self.to_string()
        } else {
            format!(
                r"{}\frac{{{}}}{{{}}}",
                if self.is_negative() { "-" } else { "" },
                self.numer().abs(),
                self.denom()
            )
        }
    }

    fn to_tex_with_positive_sign(&self) -> String {
        if self.is_integer() {
            format!(
                r"{}{}",
                if self.is_negative() { "" } else { "+" },
                self.to_string()
            )
        } else {
            format!(
                r"{}\frac{{{}}}{{{}}}",
                if self.is_negative() { "-" } else { "+" },
                self.numer().abs(),
                self.denom()
            )
        }
    }

    fn to_tex_with_paren(&self) -> String {
        if self.is_integer() {
            format!(
                r"{}{}{}",
                if self.is_negative() { r"\left(" } else { "" },
                self.to_string(),
                if self.is_negative() { r"\right)" } else { "" },
            )
        } else {
            format!(
                r"{}\frac{{{}}}{{{}}}{}",
                if self.is_negative() { r"\left(-" } else { "" },
                self.numer().abs(),
                self.denom(),
                if self.is_negative() { r"\right)" } else { "" },
            )
        }
    }

    fn to_tex_ignore_one(&self) -> String {
        if self.is_integer() {
            if self.is_one() {
                "".to_string()
            } else if self.abs().is_one() {
                "-".to_string()
            } else {
                self.to_string()
            }
        } else {
            format!(
                r"{}\frac{{{}}}{{{}}}",
                if self.is_negative() { "-" } else { "" },
                self.numer().abs(),
                self.denom()
            )
        }
    }

    fn to_tex_with_sign_ignore_one(&self) -> String {
        if self.is_integer() {
            if self.is_one() {
                "+".to_string()
            } else if self.abs().is_one() {
                "-".to_string()
            } else {
                format!(
                    r"{}{}",
                    if self.is_negative() { "" } else { "+" },
                    self.to_string()
                )
            }
        } else {
            format!(
                r"{}\frac{{{}}}{{{}}}",
                if self.is_negative() { "-" } else { "+" },
                self.numer().abs(),
                self.denom()
            )
        }
    }

    fn sign_to_tex(&self) -> String {
        if self.is_positive() {
            "".to_string()
        } else {
            "-".to_string()
        }
    }

    fn sign_to_tex_with_positive_sign(&self) -> String {
        if self.is_positive() {
            "+".to_string()
        } else {
            "-".to_string()
        }
    }
}

impl<T> ToTex for Matrix<T>
where
    T: ToTex,
{
    fn to_tex(&self) -> String {
        self.map(T::to_tex).to_string()
    }

    fn to_tex_with_positive_sign(&self) -> String {
        self.map(T::to_tex_with_positive_sign).to_string()
    }

    fn to_tex_with_paren(&self) -> String {
        self.map(T::to_tex_with_paren).to_string()
    }

    fn to_tex_ignore_one(&self) -> String {
        self.map(T::to_tex_ignore_one).to_string()
    }

    fn to_tex_with_sign_ignore_one(&self) -> String {
        self.map(T::to_tex_with_sign_ignore_one).to_string()
    }

    fn sign_to_tex(&self) -> String {
        self.map(T::sign_to_tex).to_string()
    }

    fn sign_to_tex_with_positive_sign(&self) -> String {
        self.map(T::sign_to_tex_with_positive_sign).to_string()
    }
}
