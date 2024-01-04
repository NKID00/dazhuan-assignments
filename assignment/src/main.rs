use std::panic;

use shiyanyi::Shiyanyi;

mod common;
mod discrete;
mod linalg;

fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    Shiyanyi::builder()
        .section(
            "linalg",
            "线性代数",
            Shiyanyi::builder()
                .solver_default::<linalg::InversionNumber>()
                .solver_default::<linalg::ReducedRowEchelon>(),
        )
        .section(
            "discrete",
            "离散数学",
            Shiyanyi::builder()
                .solver_default::<discrete::Exp1>()
                .solver_default::<discrete::Exp2>()
                .solver_default::<discrete::Exp3>()
                .solver_default::<discrete::Exp4>(),
        )
        .build()
        .boot("shiyanyi");
}
