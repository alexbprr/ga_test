mod ga;
mod utils;
mod data;
mod ode_solver;

use std::{fs::File, iter};
use ode_solvers::DVector;
use self::{data::CSVData, ga::GA, ode_solver::{OdeProblem, State}};

/*
    Objective: to find the parameter values that better adjust the set of experimental data. 
*/

#[derive(Default)]
pub struct ParameterEstimation {
    ga: GA,
    best_solution: Vec<f64>,
}

impl ParameterEstimation {

    pub fn new() -> Self {
        Self {
            ga: GA::default(),
            best_solution: vec![],
        }
    }

    pub fn estimate_parameters(&mut self){
        
        match CSVData::load_data(File::open("./src/tests/sir_data.csv").unwrap()){

            Ok(csv_data) => {

                let bounds: Vec<(f64,f64)> = vec![(0.0001,0.1),(0.001,1.0),(0.001,1.0)];
                self.ga = GA::new(50, 0.5, 0.5, bounds.clone(), true);
                self.ga.generate_random_population(30, bounds.len());

                const N_MAX: usize = 2; //number of available population data 

                match self.ga.optimize( |values: &Vec<f64>| {
                    
                    let t_ini = 0.0;
                    let t_final = 50.0;
                    let dt = 0.01;
                    let y0 = State::from(vec![1000.0,5.0,0.0]);                    
                    let ode_result: Vec<DVector<f64>> = OdeProblem::solve(t_ini, t_final, dt, y0, State::from((*values).clone()));
                    
                    let mut t: f64 = t_ini;
                    let mut index: usize = 0;
                    let mut errors: Vec<f64> = vec![0.0; N_MAX];
                    let mut sums: Vec<f64> = vec![0.0; N_MAX];
                    
                    for vec_values in ode_result.iter(){
                        //time_results.push((t.clone(),vec_values.clone()));
                        match csv_data.time.get(index){
                            Some(time) => {
                                if (t - time).abs() < 1e-08 {
                                    for j in 0..N_MAX {
                                        let data: f64 = csv_data.lines[j][index];
                                        errors[j] += (data - vec_values[j])*(data - vec_values[j]);
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