mod core;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::io::Write;
use std::io::BufReader;
use core::ParameterEstimation;
use core::json::{load_json, save_json, Argument, Bound, ConfigData, Metadata};

use std::env;

use crate::core::model::create_ode_system;
use crate::core::model::State;
fn main(){
    env::set_var("RUST_BACKTRACE", "1");
    
    let metadata = Metadata {name: String::from("Parameter estimation"), start_time: 0.0, 
        delta_time: 0.001, end_time: 10.0, population_size: 80, crossover_rate: 0.5, mutation_rate: 0.7};
    
    let mut arguments: Vec<Argument> = vec![];
    arguments.push(Argument::new(String::from("S"), 1000.0));
    arguments.push(Argument::new(String::from("I"), 5.0));
    arguments.push(Argument::new(String::from("R"), 0.0));
    arguments.push(Argument::new(String::from("alpha"), 0.3));
    arguments.push(Argument::new(String::from("beta"), 0.008));
    arguments.push(Argument::new(String::from("gamma"), 0.15));

    let mut config_bounds = vec![];
    config_bounds.push(Bound::new(String::from("alpha"),0.1, 1.0));
    config_bounds.push(Bound::new(String::from("beta"),0.001, 0.1));
    config_bounds.push(Bound::new(String::from("gamma"),0.1, 1.0));

    let _ = save_json(ConfigData { metadata: metadata, arguments: arguments, bounds: config_bounds}, 
            "./src/config/ga_input.json");
                
    let config_data = match load_json("./src/config/ga_input.json") {
        Ok(config_model) => {println!("Config data: {:?}", config_model); config_model },
        Err(e) => {println!("Error caused by {:?}", e); return},
    };

    let input_buffer: &mut String = &mut String::from("");
    let file: File = match File::open("./src/tests/sir.txt") {
        Ok(f) => f,
        Err(e) => {println!("Error! {:?}", e); return;},
    };
    let mut reader: BufReader<File> = BufReader::new(file);
    reader.read_to_string(input_buffer).unwrap();

    //let mut ode_system = create_ode_system(String::from("I = beta*S*I - alpha*I \n R = alpha*I - gamma*R \n S = -beta*S*I + gamma*R"), &config_data);
    let mut ode_system = create_ode_system(input_buffer.to_string(), &config_data);
    println!("ODEs: {:#?}", ode_system);

    let y: State = State::from_vec(ode_system.equations.keys()
                    .map(|k| ode_system.get_argument_value(k.to_string())).collect());
    //println!("{:#?}", ode_system.solve(y));
    
    let mut test: ParameterEstimation = ParameterEstimation::new( 
        String::from("./src/tests/sir_data.csv"), config_data, 30);
    test.estimate_parameters(&mut ode_system);
    
}