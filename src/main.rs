mod core;
use core::model::{create_ode_system, OdeSystem};
use core::ParameterEstimation;
use core::json::{load_json, save_json, Argument, Bound, ConfigData, Metadata};

use crate::core::model::{solve, State};

fn main(){
    let metadata = Metadata {name: String::from("Parameter estimation"), start_time: 0.0, delta_time: 50.0, end_time: 0.01};
    let mut arguments: Vec<Argument> = vec![];
    arguments.push(Argument::new(String::from("S"), 1000.0));
    arguments.push(Argument::new(String::from("I"), 10.0));
    arguments.push(Argument::new(String::from("R"), 0.0));
    let mut config_bounds = vec![];
    config_bounds.push(Bound::new(String::from("alpha"),0.001, 0.1));
    config_bounds.push(Bound::new(String::from("beta"),0.001, 0.1));
    config_bounds.push(Bound::new(String::from("gamma"),0.001, 0.1));

    let _ = save_json(ConfigData { metadata: metadata, 
                populations: vec!["S".to_string(), "I".to_string(), "R".to_string()],
                parameters: vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()],
                arguments: arguments, bounds: config_bounds}, "./src/config/ga_input.json");
                
    let config_data = match load_json("./src/config/ga_input.json") {
        Ok(config_model) => {println!("Config data: {:?}", config_model); config_model },
        Err(e) => {println!("Error caused by {:?}", e); return},
    };

    let bounds: Vec<(f64,f64)> = vec![(0.0001,0.1),(0.001,1.0),(0.001,1.0)];
    //TO DO: Pass config data to GA 
    let mut test: ParameterEstimation = ParameterEstimation::new(String::from("./src/tests/sir_data.csv"),
            3, bounds);
    //test.estimate_parameters(50, 0.75, 0.5);



    let test_string = String::from("H = r*H +- a*H*P \n P = a*H*P +- m*P");
    let mut ode_system = create_ode_system(test_string);
    ode_system.values.insert(String::from("H"), 100.0);
    ode_system.values.insert(String::from("P"), 5.0);
    ode_system.values.insert(String::from("r"), 0.25);
    ode_system.values.insert(String::from("a"), 0.001);
    ode_system.values.insert(String::from("m"), 0.4);

    println!("{:#?}", ode_system);
    println!("result = {:#?}", solve(ode_system, 0.0, 10.0, 0.1,State::from_vec(vec![100.0, 5.0])));
}