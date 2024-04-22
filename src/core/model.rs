use std::{cell::RefCell, collections::BTreeMap};
use mexprp::{Answer, Context, Expression};
use ode_solvers::dop853::*;
use ode_solvers::*;
use std::{fs::File, io::{BufWriter, Write}, path::Path};

use super::json::ConfigData;

pub type State = DVector<f64>;

#[derive(Debug, Clone)]
pub struct OdeSystem {
    pub equations: BTreeMap<String,(Expression<f64>, f64)>,
    pub context: Context<f64>,
    pub config_data: ConfigData,
}

impl OdeSystem {
    pub fn new(cfg: ConfigData) -> Self{
        Self {
            equations: BTreeMap::new(),
            context: Context::new(),
            config_data: cfg,
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
    
    pub fn solve(&self, y: State) -> Vec<State> {
    
        let mut solver = 
                Dop853::new(
                    self.clone(), 
                    self.config_data.metadata.start_time, 
                    self.config_data.metadata.end_time, 
                    self.config_data.metadata.delta_time, 
                    y, 
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

    pub fn update_context(&self, y: &State) {

        let equations_ref: RefCell<BTreeMap<String,(Expression<f64>, f64)>> = 
                RefCell::new(self.equations.to_owned());                
        let mut eqs = equations_ref.borrow_mut();

        let context_ref: RefCell<Context<f64>> = RefCell::new(self.context.to_owned());
        let mut context_mut = context_ref.borrow_mut();

        eqs.values_mut()
                .zip(y.iter())
                .for_each(|(current_value, new_value)| 
                    (*current_value).1 = *new_value);
        
        for (name, equation) in eqs.iter(){
            context_mut.set_var(name, equation.1);
        }

        for equation in eqs.values_mut(){
            equation.0.ctx = context_mut.clone();
        }
    }
}

pub fn create_ode_system(input: String, config_data: &ConfigData) -> OdeSystem {
        
    let mut ode_system = OdeSystem::new(config_data.clone());

    for arg in config_data.arguments.iter(){
        ode_system.context.set_var(&arg.name, arg.value);
    }
    
    let lines = input.split("\n").collect::<Vec<_>>(); //to do: testar trim 

    for line in lines {
        let new_line = 
                line
                    .trim()
                    .split('=')
                    .filter(|&s| !s.is_empty())
                    .collect::<Vec<_>>();
                
        if new_line.len() == 2 {

            let population = new_line[0].trim().to_string();            

            let ode_rhs: Expression<f64> = Expression::parse_ctx(&new_line[1].trim(), 
                            ode_system.context.clone()).unwrap();
            ode_system.equations.insert(population.clone(), 
                    (ode_rhs, ode_system.get_argument_value(population)));
        }
    }

    return ode_system
}

impl ode_solvers::System<f64, State> for OdeSystem {

    fn system(&self, _t: f64, y: &State, dydt: &mut State) {
        
        self.update_context(y);

        let mut i: usize = 0;
        for (equation, _value) in self.equations.values() {

            if let Ok(Answer::Single(expr_value)) =  equation.eval(){
                dydt[i] = expr_value;
            }
            i += 1;
        }

    }
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