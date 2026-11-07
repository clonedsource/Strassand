// Author: Vincent Dellmar
// Created: 11-07-2026 (Happy Birthday to me!)
// Email: vide1819@colorado.edu
// Description:
/*
    This is a module for using creating programs using matrlang (a custom language by me) 
    that allows you to do matrix operations in a coded file. Allows exporting data into a csv.
*/
#![allow(unused_imports)]
use std::{collections::HashMap, env::{self, Args}, fs::*, io::{BufRead, BufReader}, path::PathBuf, thread::{self, JoinHandle}};
use super::matrix::*; 

mod executor;

pub fn create_folders() {
    let path = env::current_dir();
    if path.is_err() {return;}
    let mut path_buf = path.unwrap();
    path_buf = path_buf.join("LocalData");

    let mut BobTheBuilder = std::fs::DirBuilder::new();
    // Sees if ..\LocaData exists.
    if !std::fs::exists(path_buf.clone()).unwrap() {
        if BobTheBuilder.create(path_buf.clone()).is_err() {return;}
    }

    if !std::fs::exists(path_buf.clone().join("Logs")).unwrap() {
        if BobTheBuilder.create(path_buf.clone().join("Logs")).is_err() {return;}
    }
    
    if !std::fs::exists(path_buf.clone().join("Programs")).unwrap() {
        if BobTheBuilder.create(path_buf.clone().join("Programs")).is_err() {return;}
    }

    if !std::fs::exists(path_buf.clone().join("Exports")).unwrap() {
        if BobTheBuilder.create(path_buf.clone().join("Exports")).is_err() {return;}
    }
}

pub fn run(path: PathBuf) {
    // Decompose the file with reading.
    /*
        commands:
        define VARIABLE_NAME = VALUE;
        dim MATRIX_NAME(DIM0,DIM1) = [...];
        op OPERATION_NAME(...ARGS) DESTINATION;
        call FUNCTION_NAME(...ARGS) (...DESTINATIONS);
        loop VARIABLE_NAME(START,END) ...commands_here endloop;
        export set/add VARIABLE_OR_MATRIX_NAME;
     */
    let mut prgm_file = File::open(path).unwrap();
    let mut buf_reader = BufReader::new(prgm_file);

    let mut cur_buf = vec![];
    let mut num_bytes = 50;

    let mut variables_hash_map: HashMap<String, f64> = HashMap::new();
    let mut matrices_hash_map: HashMap<String, Matrix<f64>> = HashMap::new();
    loop {
        // Important to ensure EOF
        let num_bytes = buf_reader.read_until(b';', &mut cur_buf).unwrap();
        if num_bytes == 0 {break;}
        
        // What we translate
        let mut codeline = String::from_utf8_lossy(&cur_buf).trim().to_string();
        codeline.pop();

        // Split by Spaces.
        let mut codevec: Vec<&str> = codeline.split(' ').collect();
        let opcode = codevec.remove(0);
        println!("opcode: {}",opcode);

        
        // Ugly conditional. I don't particularly care though lol.
        if opcode == "define" {
            let result: Result<(String, f64), String> = define_op(codevec);
            if result.is_err() {
                // TODO! Make this produce a line in the log.
                println!("{}", result.unwrap_err());
                break;
            }
            let (variable, value) = result.unwrap();
            variables_hash_map.insert(variable, value);
        } else if opcode == "dim" {
            let result: Result<(String, Matrix<f64>), String> = dim_op(codevec);
            if result.is_err() {
                // TODO! Make this produce a line in the log.
                println!("{}", result.unwrap_err());
                break;
            }
            let (variable, value) = result.unwrap();
            matrices_hash_map.insert(variable, value);
        } else if opcode == "op" {
            op_op(codevec, &matrices_hash_map as *const HashMap<String, Matrix<f64>>, &variables_hash_map as *const HashMap<String,f64>);
        } else if opcode == "call" {
            call_op(codevec);
        } else if opcode == "loop" {
            loop_op(codevec);
        } else if opcode == "export" {
            export_op(codevec);
        } else { // If not, produce an error in the log!
            // TODO! Make this produce a line in the log.
            break;
        }

        cur_buf.clear();
    }

}

pub fn view_log(path: PathBuf) {

}

// Functions For OpCode HashMap.
fn define_op(codevec: Vec<&str>) -> Result<(String, f64), String> {
    let parse_result = codevec[1].parse::<f64>();
    // Handles parsing error.
    if parse_result.is_err() {return Err(parse_result.unwrap_err().to_string());}
    Ok((codevec[0].to_string(), parse_result.unwrap()))
}
fn dim_op(codevec: Vec<&str>) -> Result<(String, Matrix<f64>), String> {
    // Error handling galore.
    if !codevec[0].contains('(') {
        return Err("Missing '(' in 'dim MATR(x,y)'!".to_string());
    } else if !codevec[0].contains(',') {
        return Err("Missing ',' in 'dim MATR(x,y)'!".to_string());
    } else if !codevec[0].contains(')') {
        return Err("Missing ')' in 'dim MATR(x,y)'!".to_string());
    } else if !codevec[1].contains('[') {
        return Err("Missing '[' in 'dim MATR(x,y) = [x1,x2,...,xn]'!".to_string());
    } else if !codevec[1].contains(']') {
        return Err("Missing ']' in 'dim MATR(x,y) = [x1,x2,...,xn]'!".to_string());
    }

    // Splits string into name and dimensions
    let mut splitted_dim: Vec<&str> = codevec[0].split('(').collect();
    if splitted_dim[0].is_empty() {
        return Err("Missing name in dim declaration!\n Correct form is 'dim MATR(X,Y) = [x1,x2,...,xn]'!".to_string());
    }
    let name = splitted_dim[0].to_string();

    // Splits to separate dims
    splitted_dim = splitted_dim[1].split(')').collect();
    splitted_dim = splitted_dim[0].split(',').collect();
    let (dimx_str, dimy_str) = (splitted_dim[0], splitted_dim[1]);
    
    // Parses for dim + error handling.
    let parse_result_x = dimx_str.parse::<usize>();
    if parse_result_x.is_err() {return Err(parse_result_x.unwrap_err().to_string());}
    let dimx: usize = parse_result_x.unwrap();

    let parse_result_y = dimy_str.parse::<usize>();
    if parse_result_y.is_err() {return Err(parse_result_y.unwrap_err().to_string());}
    let dimy: usize = parse_result_y.unwrap();
 
    // Create a vector
    let mut matr_vec: Vec<f64> = vec![0_f64;dimx*dimy];

    // Parses into a list (Reusing splitted dim lol)
    splitted_dim = codevec[1].split('[').collect();
    splitted_dim = splitted_dim[1].split(']').collect();
    // Make sure the matrix isn't empty.
    if splitted_dim[0].is_empty() {
        return Err("cannot use empty vector for creating a matrix in 'dim MATR(X,Y) = [x1,x2,...,xn]'!".to_string());
    }
    let list: Vec<&str> = splitted_dim[0].split(',').collect();
    for (idx, value) in list.into_iter().enumerate() {
        let parse_result_value = value.parse::<f64>();
        if parse_result_value.is_err() {return Err(parse_result_value.unwrap_err().to_string());}
        matr_vec[idx] = parse_result_value.unwrap();
    }

    Ok((name,Matrix::new((dimx, dimy), Some(matr_vec))))
}
fn op_op(codevec: Vec<&str>, matr_ptr: *const HashMap<String, Matrix<f64>>, vars_ptr: *const HashMap<String, f64>) {
    let operation = codevec[0];
    // Find a way to handle operations
}
fn call_op(codevec: Vec<&str>) {

}
fn loop_op(codevec: Vec<&str>) {
    
}
fn export_op(codevec: Vec<&str>) {
    
}