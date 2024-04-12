use std::collections::{BTreeMap, HashMap};
use mexprp::{Answer, Context, Expression};
use ode_solvers::dop853::*;
use ode_solvers::*;
use std::{fs::File, io::{BufWriter, Write}, path::Path};

pub type State = DVector<f64>;

//ODE structure for the real time Rust solver 
#[derive(Debug, Clone)]
pub struct OdeEquation {
    name: String,
    text: String, 
    expressions: Vec<String>,
    value: f64,
}

impl OdeEquation {
    pub fn new(name: String, text: String) -> Self {
        Self {
            name: name,
            text: text,
            expressions: vec![],
            value: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OdeSystem {
    pub equations: BTreeMap<String,OdeEquation>,
    pub values: BTreeMap<String,f64>, 
    pub map_index: HashMap<usize,String>,
}

impl OdeSystem {
    pub fn new() -> Self{
        Self {
            equations: BTreeMap::new(),
            values: BTreeMap::new(),
            map_index: HashMap::new(),
        }
    }        
}

pub fn create_ode_system(input: String) -> OdeSystem{
    let mut ode_system = OdeSystem::new();

    let lines = input.trim().split("\n").collect::<Vec<_>>();

    let mut index = 0;

    for line in lines {   

        let new_line = line.split("=")
                    .filter(|&s| !s.is_empty())
                    .collect::<Vec<_>>();
        let population = new_line[0].trim();
        let equation = new_line[1].trim();

        ode_system.map_index.insert(index, population.to_string());
        
        let expressions = equation.split("+")
                    .filter(|&s| !s.is_empty())
                    .collect::<Vec<_>>();                    

        let mut ode = OdeEquation::new(population.to_string(), equation.to_string());
        ode.expressions = expressions.iter().map(|&s| s.to_string()).collect::<Vec<String>>();

        ode_system.equations.insert(population.to_string(), ode);

        index += 1;
    }

    return ode_system
}

pub fn solve(system: OdeSystem, t_ini: f64, t_final: f64, dt: f64, y0: State) -> Vec<State> {
    
    let mut stepper = 
                Dop853::new(system, t_ini, t_final, dt, y0, 1.0e-8, 1.0e-8);
    let res = stepper.integrate();

    match res {
        Ok(stats) => {
            let result = stepper.y_out().to_vec();                
            return result;
        }
        Err(e) => {println!("An error occured: {}", e); return vec![]; } ,
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

    // Write time and state vector in a csv format
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

pub fn create_context(values: BTreeMap<String,f64>) -> Context<f64> {
    let mut context: Context<f64> = Context::new();
    
    for (name, value) in values.iter(){
        context.set_var(name, value.clone());
    }

    return context
}

impl ode_solvers::System<f64, State> for OdeSystem {

    fn system(&self, _t: f64, y: &State, dydt: &mut State) {

        let mut values = self.values.clone();
        let mut id: usize = 0;

        for new_value in y.iter() {
            if let Some(name) = self.map_index.get(&id){
                let value = values.get_mut(name).unwrap();
                *value = *new_value;
            }
            id += 1;
        }
        
        let context = create_context(values);
        println!("context = {:#?}", context);

        //to do: percorrer o vetor de equações (na ordem dos índices) para pegar o texto das expressões 
        for id in 0..self.equations.len() {
            if let Some(name) = self.map_index.get(&id){
                let expr: String = self.equations.get(name).unwrap().text.clone();
                let math_expr = Expression::parse_ctx(&expr, context.clone()).unwrap();
                let res: Result<mexprp::Answer<f64>, mexprp::MathError> = math_expr.eval();
                if let Ok(Answer::Single(expr_value)) = res {
                    dydt[id] = expr_value;
                }
            }         
        }
    }
}