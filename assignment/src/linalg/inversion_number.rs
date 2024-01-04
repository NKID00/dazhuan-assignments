use itertools::Itertools;
use leptos::*;
use num::{zero, BigInt, Integer, bigint::ToBigInt as _};
use shiyanyi::*;

fn inv(numbers: &[BigInt]) -> BigInt {
    let l = numbers.len();
    let mut ans = zero();
    for i in 0..l {
        for j in i..l {
            if numbers[i] > numbers[j] {
                ans += 1;
            }
        }
    }
    ans
}

#[test]
fn test_inv() {
    assert_eq!(
        inv(vec![1, 2, 3, 4]
            .into_iter()
            .map(|n| n.to_bigint().unwrap())
            .collect_vec()
            .as_slice()),
        0.to_bigint().unwrap()
    );
    assert_eq!(
        inv(vec![4, 3, 2, 1]
            .into_iter()
            .map(|n| n.to_bigint().unwrap())
            .collect_vec()
            .as_slice()),
        6.to_bigint().unwrap()
    );
    assert_eq!(
        inv((1..=1000)
            .map(|n| n.to_bigint().unwrap())
            .collect_vec()
            .as_slice()),
        0.to_bigint().unwrap()
    );
    assert_eq!(
        inv((1..=1000)
            .rev()
            .map(|n| n.to_bigint().unwrap())
            .collect_vec()
            .as_slice()),
        (1000 * 999 / 2).to_bigint().unwrap()
    );
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct InversionNumber;

impl Solver for InversionNumber {
    fn id(&self) -> String {
        "inversion-number".to_string()
    }

    fn title(&self) -> String {
        "逆序数".to_string()
    }

    fn description(&self) -> View {
        "输入整数序列.".into_view()
    }

    fn default_input(&self) -> String {
        "4 3 2 1".to_string()
    }

    fn solve(&self, input: String) -> View {
        let numbers: Vec<BigInt> = match input
            .split(|c: char| !c.is_ascii_digit())
            .map(|s| s.parse::<BigInt>())
            .try_collect()
        {
            Ok(v) => v,
            Err(_) => {
                return "Failed to parse.".into_view();
            }
        };
        if numbers.is_empty() {
            return "Input is empty.".into_view();
        }
        let inversion_number = inv(&numbers[..]);
        let numbers = numbers.into_iter().join(r" \allowbreak\  ");
        view! {
            <div class="mb-10">
                <p class="font-bold mb-2"> "逆序数" </p>
                <KaTeX expr={ format!(r"\tau({numbers}) = {inversion_number} ") } />
            </div>
            <div class="mb-10">
                <p class="font-bold mb-2"> "序列类型" </p>
                <p> { if inversion_number.is_odd() { "奇序列" } else { "偶序列" }} </p>
            </div>
        }
        .into_view()
    }
}
