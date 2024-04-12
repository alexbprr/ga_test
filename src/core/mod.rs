mod ga;
mod utils;
mod data;
mod ode_solver;
pub mod json;
pub mod model;

use std::fs::File;
use ode_solvers::DVector;
use self::{data::CSVData, ga::GA, ode_solver::{OdeProblem, State}};

/*
    Objective: to find the parameter values that better adjust the set of experimental data. 
*/

pub struct ParameterEstimation {
    ga: GA,
    best_solution: Vec<f64>,
    data_file: String,
    num_of_params: usize, //number of parameters to be adjusted 
    bounds: Vec<(f64,f64)>,    
}

impl ParameterEstimation {

    pub fn new(file_name: String, n_params: usize, bounds: Vec<(f64,f64)>) -> Self {
        Self {
            ga: GA::default(),
            best_solution: vec![],
            data_file: file_name,
            num_of_params: n_params,
            bounds: bounds,
        }
    }

    pub fn estimate_parameters(&mut self, p_size: usize, mut_rate: f64, cross_rate: f64){
        
        match CSVData::load_data(File::open(self.data_file.clone()).unwrap()){

            Ok(csv_data) => {
                        
                self.ga = GA::new(p_size, mut_rate, cross_rate, self.bounds.clone(),
                            true);

                self.ga.generate_random_population(p_size, self.num_of_params);

                let n_max: usize = csv_data.labels.len(); //number of available population data 

                match self.ga.optimize( |values: &Vec<f64>| {
                    
                    let t_ini = 0.0;
                    let t_final = 50.0;
                    let dt = 0.01;
                    let y0 = State::from(vec![1000.0,5.0,0.0]);                    
                    let ode_result: Vec<DVector<f64>> = OdeProblem::solve(t_ini, t_final, dt, y0, 
                                State::from((*values).clone()));
                    
                    let mut t: f64 = t_ini;
                    let mut index: usize = 0;
                    let mut errors: Vec<f64> = vec![0.0; n_max];
                    let mut sums: Vec<f64> = vec![0.0; n_max];
                    
                    for ode_values in ode_result.iter(){
                        
                        match csv_data.time.get(index){

                            Some(time) => {
                                if (t - time).abs() < 1e-08 {
                                    for j in 0..n_max {
                                        let data: f64 = csv_data.lines[j][index];
                                        errors[j] += (data - ode_values[j])*(data - ode_values[j]);
                                        sums[j] += data*data;
                                    } 
                                    index += 1;
                                }                                
                            },
                            None => break,
                        }                        

                        t += dt;
                    }
                    
                    let error = 
                        errors
                        .iter()
                        .zip(sums.iter())
                        .map(|v: (&f64, &f64)| v.0/v.1)                        
                        .collect::<Vec<f64>>()
                        .iter()
                        .sum::<f64>()
                        .sqrt(); 

                    return error;
                } ){                    
                    Ok(c) => { println!("The best individual is {:?}", c); self.best_solution = c.get_values(); },
                    Err(e) => println!("An error ocurred {:?}", e),
                }                    
            },
            Err(e) => println!("An error ocurred: {:?}", e),
        }

    }

}