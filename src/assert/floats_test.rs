use super::floats::*;

#[test]
fn in_delta_f64_ok() {
    in_delta_f64(1.0, 1.001, 0.01);
}

#[test]
#[should_panic(expected = "`in_delta_f64`")]
fn in_delta_f64_fails() {
    in_delta_f64(1.0, 2.0, 0.1);
}

#[test]
fn in_delta_f32_ok() {
    in_delta_f32(1.0_f32, 1.01_f32, 0.1);
}

#[test]
fn in_delta_slice_f64_ok() {
    in_delta_slice_f64(&[1.0, 2.0], &[1.001, 2.001], 0.01);
}

#[test]
fn in_epsilon_f64_ok() {
    in_epsilon_f64(100.0, 100.0, 1e-9);
}

#[test]
fn in_epsilon_slice_f64_ok() {
    in_epsilon_slice_f64(&[1.0, 2.0], &[1.0, 2.0], 1e-9);
}
