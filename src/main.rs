mod ode;
use ode::ParameterEstimation;
use ode::ga_json::{save_json, GA_Argument, Bound, ConfigData, GA_Metadata};

use std::env;

fn main(){
    env::set_var("RUST_BACKTRACE", "1");
    
    let metadata = GA_Metadata {name: String::from("Parameter estimation"), start_time: 0.0, 
        delta_time: 0.1, end_time: 60.0, population_size: 80, crossover_rate: 0.5, mutation_rate: 0.8, max_iterations: 50};
    
    let mut arguments: Vec<GA_Argument> = vec![];
    arguments.push(GA_Argument::new(String::from("N"), 1.));
    arguments.push(GA_Argument::new(String::from("r"), 0.1));
    arguments.push(GA_Argument::new(String::from("k"), 50.));

    let mut config_bounds = vec![];
    config_bounds.push(Bound::new(String::from("r"),0.1, 1.0));
    config_bounds.push(Bound::new(String::from("k"),1.0, 200.));

    let _ = save_json(ConfigData { metadata: metadata, arguments: arguments, bounds: config_bounds}, 
            "./src/ode/config/ga_input.json");
    
    let mut param_estimator: ParameterEstimation = ParameterEstimation::new( 
        String::from("./src/ode/tests/logistic_data.csv"));
    let mut ode_system = param_estimator.ode_system("./src/ode/config/ga_input.json", "./src/ode/tests/logistic.txt");
    param_estimator.estimate_parameters(&mut ode_system);
    
}