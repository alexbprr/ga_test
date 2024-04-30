use std::collections::BTreeMap;
use mexprp::{Answer, Context, Expression};

use ode_solvers::*;
use std::{fs::File, io::{BufWriter, Write}, path::Path};

use super::ga_json::ConfigData;

pub type State = DVector<f64>;

#[derive(Debug, Clone)]
pub struct OdeSystem {
    pub config_data: ConfigData,
    pub equations: BTreeMap<String,(Expression<f64>, f64)>,
    pub context: Context<f64>,
}

impl OdeSystem {
    pub fn new(cfg: ConfigData) -> Self{
        Self {
            config_data: cfg,
            equations: BTreeMap::new(),
            context: Context::new(),            
        }
    }

    pub fn get_argument_value(&self, name: String) -> f64{
        for arg in self.config_data.arguments.iter() {
            if arg.name == name {
                return arg.value
            }        
        }
        return 0.0
    }

    pub fn update_context(&mut self, values: &Vec<f64>) {

        self.config_data.bounds
                .iter()
                .zip(values)
                .for_each(|(bound,value)| 
                    self.context.set_var(&bound.name, *value)
                );
    }

    pub fn update_context_with_state(&mut self, y: &State) {

        self.equations
                .values_mut()
                .zip(y.iter())
                .for_each(|(current_value, new_value)| 
                    (*current_value).1 = *new_value);
        
        for (name, expr) in self.equations.iter(){
            self.context.set_var(name, expr.1);
        }
    }
}

impl ode_solvers::System<f64, State> for OdeSystem {

    fn system(&mut self, _t: f64, y: &State, dydt: &mut State) {
        
        self.update_context_with_state(y);
        //println!("context: {:#?}", self.context);

        let mut i: usize = 0;
        for (equation, _value) in self.equations.values() {            

            if let Ok(Answer::Single(expr_value)) =  equation.eval_ctx(&self.context){
                dydt[i] = expr_value;
            }
            i += 1;
        }

    }
}

pub fn solve(ode_system: &mut OdeSystem, y: &State) -> Vec<State> {
    let t_ini = ode_system.config_data.metadata.start_time; 
    let t_final = ode_system.config_data.metadata.end_time; 
    let dt = ode_system.config_data.metadata.delta_time; 

    let mut solver = 
            Dop853::new(
                (*ode_system).clone(), 
                t_ini,
                t_final,
                dt,
                y.clone(), 
                1.0e-8, 
                1.0e-8);
    
    match solver.integrate() {
        Ok(_stats) => {
            return solver.y_out().to_vec();
        }
        Err(e) => {
            println!("An error occured: {}", e);
            return vec![]; 
        },
    }
}

pub fn create_ode_system(input: String, config_data: &ConfigData) -> OdeSystem {
        
    let mut ode_system = OdeSystem::new(config_data.clone());       
    
    let lines = input.split("\n").collect::<Vec<_>>(); 

    for line in lines {
        let new_line = 
                line
                    .trim()
                    .split('=')
                    .filter(|&s| !s.is_empty())
                    .collect::<Vec<_>>();
                
        if new_line.len() == 2 {

            let population = new_line[0].trim().to_string();            

            let ode_rhs: Expression<f64> = Expression::parse(&new_line[1].trim()).unwrap();

            ode_system.equations.insert(population.clone(), 
                    (ode_rhs, ode_system.get_argument_value(population)));
        }
    }

    for arg in config_data.arguments.iter(){     
        ode_system.context.set_var(&arg.name.trim().to_string(), arg.value);
    } 

    return ode_system
}

pub fn save(times: &Vec<f64>, states: &Vec<State>, filename: &Path) {
    // Create or open file
    let file = match File::create(filename) {
        Err(e) => {
            println!("Could not open file. Error: {:?}", e);
            return;
        }
        Ok(buf) => buf,
    };
    let mut buf = BufWriter::new(file);

    // Write time and state vector in csv format
    for (i, state) in states.iter().enumerate() {
        if i >= times.len() {
            break;
        }
        buf.write_fmt(format_args!("{:.6}", times[i])).unwrap();
        for val in state.iter() {
            buf.write_fmt(format_args!(", {}", val)).unwrap();
        }
        buf.write_fmt(format_args!("\n")).unwrap();
    }
    if let Err(e) = buf.flush() {
        println!("Could not write to file. Error: {:?}", e);
    }
}