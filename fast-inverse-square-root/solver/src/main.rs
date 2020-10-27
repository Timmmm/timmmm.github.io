use argmin::prelude::*;
use argmin::solver::neldermead::NelderMead;

#[derive(Clone, Default, Debug)]
struct InverseSqrt {}

impl ArgminOp for InverseSqrt {
    type Param = Vec<f64>;
    type Output = f64;
    type Hessian = ();
    type Jacobian = ();
    type Float = f64;

    fn apply(&self, p: &Self::Param) -> Result<Self::Output, Error> {
        Ok(max_error(p[0], p[1], p[2]))
    }
}

fn fast_inverse_sqrt(input: f32, subtractant: u32) -> f32 {

    // Reinterpret the bits as an integer.
    let input_int: u32 = u32::from_le_bytes(input.to_le_bytes());
    // Perform the magic.
    let output_int = subtractant - (input_int >> 1);
    // Reinterpret the bits as a float.
    let output = f32::from_le_bytes(output_int.to_le_bytes());

    output
}

fn newton_raphson(input: f32, x0: f32, three_halves: f32, one_half: f32) -> f32 {
    x0 * (three_halves - (input * one_half * x0 * x0))
}

// Return the maximum absolute error, as a ratio. In other words if the correct
// answer is 1.0, then 0.5 and 2.0 are equally bad.
fn max_error(subtractant: f64, three_halves: f64, one_half: f64) -> f64 {

    // We only need to test from 1 to 4 because the error pattern repeats
    // after that.
    let err = (10000..=40000).map(|step| {
        let input = step as f32 / 10000f32;

        // True output.
        let x: f64 = (input as f64).pow(-0.5);
        // Approximate output.
        let x0 = fast_inverse_sqrt(input, (subtractant * 0x800000 as f64) as u32);
        let x1 = newton_raphson(input, x0, three_halves as f32, one_half as f32);

        // Convert to f64 for extra division precision (maybe).
        let x1 = x1 as f64;

        // The optimisation can cheat using negative numbers if we don't
        // do this.
        let x1 = x1.max(0.00001);

        (x1 / x).max(x / x1)
    }).fold(1.0f64, |a, b| a.max(b));

    // Convert it to a percentage for easy comparison.
    (err - 1.0) * 100.0
}


fn main() -> Result<(), argmin::core::Error> {

    let op = InverseSqrt{};

    // Set up solver -- note that the proper choice of the vertices is very important!
    let solver = NelderMead::new()
        .with_initial_params(vec![
            vec![190.3, 1.4, 0.4],
            vec![190.3, 1.4, 0.6],
            vec![190.3, 1.6, 0.4],
            vec![190.3, 1.6, 0.6],
            vec![190.7, 1.6, 0.4],
            vec![190.7, 1.6, 0.6],
            vec![190.7, 1.4, 0.4],
            vec![190.7, 1.4, 0.6],
        ])
        .sd_tolerance(0.0000000001);

    // Run solver
    let res = Executor::new(op, solver, vec![])
        .max_iters(1000)
        .run()?;

    // Print result
    println!("\n{}", res);

    // Print the parameters in code form.
    println!("Subtractant: 0x{:x}", (res.state.best_param[0] * 0x800000 as f64) as u32);
    println!("Three halves: {}", res.state.best_param[1] as f32);
    println!("One half: {}", res.state.best_param[2] as f32);

    Ok(())
}
