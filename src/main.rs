mod core;
use core::ParameterEstimation;
use std::env;

fn main(){
    env::set_var("RUST_BACKTRACE", "1");
    let mut test: ParameterEstimation = ParameterEstimation::new();
    test.estimate_parameters();
}