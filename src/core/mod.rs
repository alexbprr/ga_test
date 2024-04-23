mod ga;
mod utils;
mod csvdata;
pub mod json;
pub mod model;

use std::{collections::BTreeMap, fs::File, path::Path};
use ode_solvers::DVector;
use self::{csvdata::CSVData, ga::GA, json::{Bound, ConfigData}, model::{save, OdeSystem, State}};
/* Objective: to find the parameter values that better adjust the set of experimental data. */

#[derive(Debug,Clone)]
pub struct ParameterEstimation {
    ga: GA,
    best_solution: Vec<f64>,
    data_file: String,
    config_data: ConfigData, 
    max_num_iterations: usize,
}

impl ParameterEstimation {

    pub fn new(file_name: String, config_data: ConfigData, max_iter: usize) -> Self {
        Self {
            ga: GA::default(),
            best_solution: vec![],
            data_file: file_name,
            config_data: config_data,
            max_num_iterations: max_iter,
        }
    }
    //TO DO: create a thread to optimize the parameters values 
    //the config input file can not be changed during execution of this ga instance
                
    pub fn estimate_parameters(&mut self, ode_system: &mut OdeSystem){
        
        match CSVData::load_data(File::open(self.data_file.clone()).unwrap()){
            Ok(csv_data) => {

                let mut bounds: BTreeMap<String,Bound> = BTreeMap::new();
                for bound in self.config_data.bounds.iter() {
                    bounds.insert(bound.name.clone(), bound.clone());
                }                

                self.ga = GA::new(
                    self.max_num_iterations, 
                    self.config_data.metadata.mutation_rate, 
                    self.config_data.metadata.crossover_rate, 
                    self.config_data.bounds.clone(),
                    true
                );

                self.ga.generate_random_population(
                    self.config_data.metadata.population_size, 
                    self.config_data.bounds.len()
                );                
                                
                match self.ga.optimize( |values: &Vec<f64>| {

                    let y: State = State::from_vec(ode_system.equations.keys()
                            .map(|k| ode_system.get_argument_value(k.to_string())).collect());

                    let mut errors: Vec<f64> = vec![0.0; csv_data.labels.len()];
                    let mut sums: Vec<f64> = vec![0.0; csv_data.labels.len()];

                    bounds.iter()
                        .zip(values.iter())
                        .for_each(|tuple| {
                            ode_system.context.set_var(tuple.0.0, *tuple.1);
                        });

                    //println!("context: {:#?}", ode_system.context);
                    
                    let ode_result: Vec<DVector<f64>> = ode_system.solve(y.clone());
                    save(&csv_data.time, &ode_result, Path::new(&String::from("./src/tests/sir_results.txt")));

                    let mut index: usize = 0;
                    let mut ode_index: usize = 0;
                    let mut t: f64 = self.config_data.metadata.start_time;

                    while t <= self.config_data.metadata.end_time {
                                                
                        if index == csv_data.time.len() {
                            break;
                        }

                        if  (t - csv_data.time[index]).abs() < 1.0_f64.powf(-self.config_data.metadata.delta_time) {

                            for i in 0..csv_data.labels.len() {

                                let data: f64 = csv_data.lines[i][index];
                                sums[i] += data*data;

                                if i == 1 {
                                    let dif = ode_result[ode_index][2] - data;
                                    errors[i] += dif*dif;
                                }
                                else {
                                    let dif = ode_result[ode_index][i] - data;
                                    errors[i] += dif*dif;
                                }        
                            }
                            index += 1;                            
                        }

                        t += self.config_data.metadata.delta_time;
                        ode_index += 1;
                    }

                    let mut id_sum: usize = 0;
                    let mut sum: f64 = 0.0;

                    for err in errors.iter(){                        
                        sum += err/sums[id_sum];
                        if sum.is_nan(){
                            return 1000.0;
                        }
                        id_sum += 1;
                    }
                    
                    return sum.sqrt();
                } ){                    
                    Ok(c) => { println!("The best individual is {:?}", c); self.best_solution = c.get_values(); },
                    Err(e) => println!("An error ocurred during the optimization: {:?}", e),
                }                    
            },
            Err(e) => println!("An error ocurred on reading the CSV file: {:?}", e),
        }

    }

}