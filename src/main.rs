mod core;
use core::model::{create_ode_system};
use core::ParameterEstimation;
use core::json::{load_json, save_json, Argument, Bound, ConfigData, Metadata};
use crate::core::model::{State};

use std::env;
fn main(){
    env::set_var("RUST_BACKTRACE", "1");
    
    let metadata = Metadata {name: String::from("Parameter estimation"), start_time: 0.0, 
        delta_time: 0.01, end_time: 20.0, population_size: 80, crossover_rate: 0.5, mutation_rate: 0.7};
    
    let mut arguments: Vec<Argument> = vec![];
    arguments.push(Argument::new(String::from("S"), 1000.0));
    arguments.push(Argument::new(String::from("I"), 5.0));
    arguments.push(Argument::new(String::from("R"), 0.0));
    arguments.push(Argument::new(String::from("alpha"), 0.3));
    arguments.push(Argument::new(String::from("beta"), 0.008));
    arguments.push(Argument::new(String::from("gamma"), 0.15));

    let mut config_bounds = vec![];
    config_bounds.push(Bound::new(String::from("alpha"),0.001, 1.0));
    config_bounds.push(Bound::new(String::from("beta"),0.001, 1.0));
    config_bounds.push(Bound::new(String::from("gamma"),0.001, 1.0));

    let _ = save_json(ConfigData { metadata: metadata, arguments: arguments, bounds: config_bounds}, 
            "./src/config/ga_input.json");
                
    let config_data = match load_json("./src/config/ga_input.json") {
        Ok(config_model) => {println!("Config data: {:?}", config_model); config_model },
        Err(e) => {println!("Error caused by {:?}", e); return},
    };

    let mut ode_system = create_ode_system(String::from("I = beta*S*I - alpha*I \n 
    R = alpha*I - gamma*R \n S = -beta*S*I + gamma*R"), &config_data);
    println!("ODEs: {:#?}", ode_system);

    let y: State = State::from_vec(ode_system.equations.keys()
                    .map(|k| ode_system.get_argument_value(k.to_string())).collect());
    //println!("{:#?}", ode_system.solve(y));
    
    let mut test: ParameterEstimation = ParameterEstimation::new( 
        String::from("./src/tests/sir_data.csv"), config_data, 50);    
    test.estimate_parameters(&mut ode_system);
    
}