// Author: Vincent Dellmar
// Created: 11-07-2026 (Happy Birthday to me!)
// Email: vide1819@colorado.edu
// Description:
/*
    This is a module for using creating programs using matrlang (a custom language by me) 
    that allows you to do matrix operations in a coded file. Allows exporting data into a csv.
*/
#![allow(unused_imports)]
use std::{collections::HashMap, env::{self, Args, vars}, fs::*, io::{BufRead, BufReader}, path::PathBuf, result, thread::{self, JoinHandle}};
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
        op OPERATION_NAME(...ARGS) MATRIX/VARIABLE_NAME;
        call FUNCTION_NAME(...ARGS) (...MATRIX/VARIABLE_NAME(S));
        loop (START..END,STEP) ...commands_here endloop;
        export set/add VARIABLE_OR_MATRIX_NAME;
        import FILE_PATH;
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
            let result: Result<String, String> = op_op(codevec, &mut matrices_hash_map, &mut variables_hash_map);
            if result.is_err() {
                // TODO! Make this produce a line in the log.
                println!("{}", result.unwrap_err());
                break;
            }
            for (key, value) in matrices_hash_map.iter() {
                println!("op {}:", key);
                value.print((false,false));
            }
        } else if opcode == "call" {
            let result: Result<String, String> = call_op(codevec, &mut matrices_hash_map, &mut variables_hash_map);
            if result.is_err() {
                // TODO! Make this produce a line in the log.
                println!("{}", result.unwrap_err());
                break;
            }
        } else if opcode == "loop" {
            loop_op(codevec);
        } else if opcode == "export" {
            export_op(codevec);
        } else if opcode == "import" { // Arguably the hardest thing to do.
            import_op(codevec);
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
fn op_op(codevec: Vec<&str>, matr_ptr: &mut HashMap<String, Matrix<f64>>, vars_ptr: &mut HashMap<String, f64>) -> Result<String, String> {
    let operation = codevec[0];
    // Inner functions
    let is_matr = |ptr: &mut HashMap<String, Matrix<f64>>, value: &str| -> bool {ptr.contains_key(value)};
    let is_var = |ptr: &mut HashMap<String, f64>, value: &str| -> bool {ptr.contains_key(value)};
    
    
    let mut add = |matr_ptr: &mut HashMap<String, Matrix<f64>>, vars_ptr: &mut HashMap<String, f64>| -> Result<String, String> {
        let object1 = codevec[1];
        let object2 = codevec[2];
        let result_name = codevec[3];
        // Make sure it even exists in either.
        if !is_matr(matr_ptr, object1) & !is_var(vars_ptr, object1) {
            println!("Undeclared variable '{}' used!", object1);
            return Err(format!("Undeclared variable '{}' used!", object1).to_string());
        }
        if !is_matr(matr_ptr, object2) & !is_var(vars_ptr, object2) {
            println!("Undeclared variable '{}' used!", object2);
            return Err(format!("Undeclared variable '{}' used!", object2).to_string());
        }

        // XOR -> Object Types Must Match.
        if is_matr(matr_ptr, object1) ^ is_matr(matr_ptr, object2) {
            println!("Object types must match under add op!");
            return Err("Object types must match under add op!".to_string());
        };
        // If they are both matrices, make sure they are both the same dimension
        if is_matr(matr_ptr, object1) {
            let matr_obj1 = matr_ptr.get(object1).unwrap();
            let matr_obj2 = matr_ptr.get(object2).unwrap();
            if matr_obj1.dim != matr_obj2.dim {
                println!("Dimension of '{}' doesn't match Dimension of '{}'!", object1, object2);
                return Err(format!("Dimension of '{}' doesn't match Dimension of '{}'!", object1, object2));
            }
            let result_matr_obj = matr_obj1 + matr_obj2;
            if matr_ptr.contains_key(result_name) {
                matr_ptr.remove(result_name);
            }
            matr_ptr.insert(result_name.to_string(), result_matr_obj);
        } else {
            let var_obj1 = vars_ptr.get(object1).unwrap();
            let var_obj2 = vars_ptr.get(object2).unwrap();

            let result_var_obj = var_obj1 + var_obj2;
            if vars_ptr.contains_key(result_name) {
                vars_ptr.remove(result_name);
            }
            vars_ptr.insert(result_name.to_string(), result_var_obj);
        };
        Ok(String::new())
    };
    let mut subtract = |matr_ptr: &mut HashMap<String, Matrix<f64>>, vars_ptr: &mut HashMap<String, f64>| -> Result<String, String> {
        let object1 = codevec[1];
        let object2 = codevec[2];
        let result_name = codevec[3];
        // Make sure it even exists in either.
        if !is_matr(matr_ptr, object1) & !is_var(vars_ptr, object1) {
            println!("Undeclared variable '{}' used!", object1);
            return Err(format!("Undeclared variable '{}' used!", object1).to_string());
        }
        if !is_matr(matr_ptr, object2) & !is_var(vars_ptr, object2) {
            println!("Undeclared variable '{}' used!", object2);
            return Err(format!("Undeclared variable '{}' used!", object2).to_string());
        }

        // XOR -> Object Types Must Match.
        if is_matr(matr_ptr, object1) ^ is_matr(matr_ptr, object2) {
            println!("Object types must match under add op!");
            return Err("Object types must match under add op!".to_string());
        };
        // If they are both matrices, make sure they are both the same dimension
        if is_matr(matr_ptr, object1) {
            let matr_obj1 = matr_ptr.get(object1).unwrap();
            let matr_obj2 = matr_ptr.get(object2).unwrap();
            if matr_obj1.dim != matr_obj2.dim {
                println!("Dimension of '{}' doesn't match Dimension of '{}'!", object1, object2);
                return Err(format!("Dimension of '{}' doesn't match Dimension of '{}'!", object1, object2));
            }
            let result_matr_obj = matr_obj1 - matr_obj2;
            if matr_ptr.contains_key(result_name) {
                matr_ptr.remove(result_name);
            }
            matr_ptr.insert(result_name.to_string(), result_matr_obj);
        } else {
            let var_obj1 = vars_ptr.get(object1).unwrap();
            let var_obj2 = vars_ptr.get(object2).unwrap();

            let result_var_obj = var_obj1 - var_obj2;
            if vars_ptr.contains_key(result_name) {
                vars_ptr.remove(result_name);
            }
            vars_ptr.insert(result_name.to_string(), result_var_obj);
        };
        Ok(String::new())
    };
    let mut multiply = |matr_ptr: &mut HashMap<String, Matrix<f64>>, vars_ptr: &mut HashMap<String, f64>| -> Result<String, String> {
        let object1 = codevec[1];
        let object2 = codevec[2];
        let result_name = codevec[3];
        // Make sure it even exists in either.
        if !is_matr(matr_ptr, object1) & !is_var(vars_ptr, object1) {
            println!("Undeclared variable '{}' used!", object1);
            return Err(format!("Undeclared variable '{}' used!", object1).to_string());
        }
        if !is_matr(matr_ptr, object2) & !is_var(vars_ptr, object2) {
            println!("Undeclared variable '{}' used!", object2);
            return Err(format!("Undeclared variable '{}' used!", object2).to_string());
        }

        // AND -> Object Types Must NOT both be matrices(unless they are both floats).
        if is_matr(matr_ptr, object1) & is_matr(matr_ptr, object2) {
            println!("Multiplying two matrices not supported under op!");
            return Err("Multiplying two matrices not supported under op!".to_string());
        };
        // If they are both matrices, make sure they are both the same dimension
        if is_matr(matr_ptr, object1) {
            let matr_obj1 = matr_ptr.get(object1).unwrap();
            let var_obj2 = vars_ptr.get(object2).unwrap();
            
            let result_matr_obj = matr_obj1 * var_obj2.clone();
            if matr_ptr.contains_key(result_name) {
                matr_ptr.remove(result_name);
            }
            matr_ptr.insert(result_name.to_string(), result_matr_obj);

        } else if is_matr(matr_ptr, object2) {
            let var_obj1 = vars_ptr.get(object1).unwrap();
            let matr_obj2 = matr_ptr.get(object2).unwrap();
            
            let result_matr_obj = matr_obj2 * var_obj1.clone();
            if matr_ptr.contains_key(result_name) {
                matr_ptr.remove(result_name);
            }
            matr_ptr.insert(result_name.to_string(), result_matr_obj);
        } else {
            let var_obj1 = vars_ptr.get(object1).unwrap();
            let var_obj2 = vars_ptr.get(object2).unwrap();

            let result_var_obj = var_obj1 * var_obj2;
            if vars_ptr.contains_key(result_name) {
                vars_ptr.remove(result_name);
            }
            vars_ptr.insert(result_name.to_string(), result_var_obj);
        };
        Ok(String::new())
    };
    let mut divide = |matr_ptr: &mut HashMap<String, Matrix<f64>>, vars_ptr: &mut HashMap<String, f64>| -> Result<String, String> {
        let object1 = codevec[1];
        let object2 = codevec[2];
        let result_name = codevec[3];
        // Make sure it even exists in either.
        if !is_matr(matr_ptr, object1) & !is_var(vars_ptr, object1) {
             println!("Undeclared variable '{}' used!", object1);
            return Err(format!("Undeclared variable '{}' used!", object1).to_string());
        }
        if !is_matr(matr_ptr, object2) & !is_var(vars_ptr, object2) {
            println!("Undeclared variable '{}' used!", object2);
            return Err(format!("Undeclared variable '{}' used!", object2).to_string());
        }

        // AND -> Object Types Must NOT both be matrices(unless they are both floats).
        if is_matr(matr_ptr, object1) & is_matr(matr_ptr, object2) {
            println!("Dividing two matrices not supported under op!");
            return Err("Dividing two matrices not supported under op!".to_string());
        };
        // If they are both matrices, make sure they are both the same dimension
        if is_matr(matr_ptr, object1) {
            let matr_obj1 = matr_ptr.get(object1).unwrap();
            let var_obj2 = vars_ptr.get(object2).unwrap();
            
            let result_matr_obj = matr_obj1 * (1_f64 / var_obj2);
            if matr_ptr.contains_key(result_name) {
                matr_ptr.remove(result_name);
            }
            matr_ptr.insert(result_name.to_string(), result_matr_obj);

        } else if is_matr(matr_ptr, object2) {
            let var_obj1 = vars_ptr.get(object1).unwrap();
            let matr_obj2 = matr_ptr.get(object2).unwrap();
            
            let result_matr_obj = matr_obj2 * (1_f64 / var_obj1);
            if matr_ptr.contains_key(result_name) {
                matr_ptr.remove(result_name);
            }
            matr_ptr.insert(result_name.to_string(), result_matr_obj);

        } else {
            let var_obj1 = vars_ptr.get(object1).unwrap();
            let var_obj2 = vars_ptr.get(object2).unwrap();

            let result_var_obj = var_obj1 / var_obj2;
            if vars_ptr.contains_key(result_name) {
                vars_ptr.remove(result_name);
            }
            vars_ptr.insert(result_name.to_string(), result_var_obj);
        };
        Ok(String::new())
    };

    if operation == "add" {
        return add(matr_ptr, vars_ptr);
    } else if operation == "subtract" {
        return subtract(matr_ptr, vars_ptr);
    } else if operation == "multiply" {
        return multiply(matr_ptr, vars_ptr);
    } else if operation == "divide" {
        return divide(matr_ptr, vars_ptr);
    } else {
        Err("Operation does not exist".to_string())
    }

    // Find a way to handle operations
}
fn call_op(codevec: Vec<&str>, matr_ptr: &mut HashMap<String, Matrix<f64>>, vars_ptr: &mut HashMap<String, f64>) -> Result<String, String> {
    if !codevec[0].contains('(') {
        return Err("Missing '(' in 'call FN(...)'!".to_string());
    } else if !codevec[0].contains(')') {
        return Err("Missing ')' in 'call FN(...)'!".to_string());
    }
    let mut args: Vec<&str> = codevec[0].split('(').collect();
    let operation = args[0];
    args = args[1].split(')').collect();
    args = args[0].split(',').collect();
    // Inner functions
    let is_matr = |ptr: &mut HashMap<String, Matrix<f64>>, value: &str| -> bool {ptr.contains_key(value)};
    let is_var = |ptr: &mut HashMap<String, f64>, value: &str| -> bool {ptr.contains_key(value)};
    
    
    let mut print = |matr_ptr: &mut HashMap<String, Matrix<f64>>, vars_ptr: &mut HashMap<String, f64>| -> Result<String, String> {
        if args.len() < 1 {
            println!("Not enough arguments passed into print!");
            return Err("Not enough arguments passed into print!".to_string());
        }
        let object1 = args[0];

        // Print matrix
        if is_matr(matr_ptr, object1) {
            let matr_obj1 = matr_ptr.get(object1).unwrap();
            if args.len() < 3 {
                println!("Not enough arguments passed into print for matrix");
                return Err("Not enough arguments passed into print for matrix!".to_string());
            }
            let arg1_wrapped = args[1].parse::<bool>();
            let arg2_wrapped = args[2].parse::<bool>();
            if arg1_wrapped.is_err() {
                println!("Argument 1 of print for matrix is wrong; expected 'print(matr,bool,bool)'!");
                return Err("Argument 1 of print for matrix is wrong; expected 'print(matr,bool,bool)'!".to_string())
            }
            if arg2_wrapped.is_err() {
                println!("Argument 2 of print for matrix is wrong; expected 'print(matr,bool,bool)'!");
                return Err("Argument 2 of print for matrix is wrong; expected 'print(matr,bool,bool)'!".to_string())
            }
            let (arg1, arg2) = (arg1_wrapped.unwrap(), arg2_wrapped.unwrap());
            matr_obj1.print((arg1, arg2));
            Ok(matr_obj1.matrix_str())

        } else if is_var(vars_ptr, object1) {
            if args.len() < 1 {
                println!("Not enough arguments passed into print for float!");
                return Err("Not enough arguments passed into print for float!".to_string());
            }
            let var_obj1 = vars_ptr.get(object1).unwrap();
            println!("{}",var_obj1);
            Ok(var_obj1.to_string())
        } else {
            println!("{}", object1);
            Ok(object1.to_string())
        }
        
    };

    let mut nmulti = |matr_ptr: &mut HashMap<String, Matrix<f64>>, vars_ptr: &mut HashMap<String, f64>| -> Result<String, String> {
        if codevec.len() < 3 {
            println!("Not enough result arguments passed into naive multiplication!");
            return Err("Not enough result arguments passed into naive multiplication!".to_string());
        }
        let (result_matr_name, result_multi_count_name) = (codevec[1], codevec[2]);
        if args.len() < 2 {
            println!("Not enough arguments passed into naive multiplication!");
            return Err("Not enough arguments passed into naive multiplication!".to_string());
        }
        let object1 = args[0];
        let object2 = args[1];
    
        // Make sure it even exists in either.
        if !(is_matr(matr_ptr, object1) & is_matr(matr_ptr, object2)) {
            println!("Not enough matrices exist!");
            return Err("Not enough matrices exist!".to_string());
        }

        // Multiply matrix
       
        let matr_obj1 = matr_ptr.get(object1).unwrap();
        let matr_obj2 = matr_ptr.get(object2).unwrap();
        let (res, multi) = naive_method(matr_obj1, matr_obj2);
        if res.is_err() {
            return Err(res.unwrap_err().to_string());
        }

        if vars_ptr.contains_key(result_multi_count_name) {
                vars_ptr.remove(result_multi_count_name);
        }
        vars_ptr.insert(result_multi_count_name.to_string(), multi as f64);
        matr_ptr.insert(result_matr_name.to_string(), res.unwrap());
        Ok(String::new())
        
    };

    let mut smulti = |matr_ptr: &mut HashMap<String, Matrix<f64>>, vars_ptr: &mut HashMap<String, f64>| -> Result<String, String> {
        if codevec.len() < 3 {
            println!("Not enough result arguments passed into strassen multiplication!");
            return Err("Not enough result arguments passed into strassen multiplication!".to_string());
        }
        let (result_matr_name, result_multi_count_name) = (codevec[1], codevec[2]);
        if args.len() < 2 {
            println!("Not enough arguments passed into strassen multiplication!");
            return Err("Not enough arguments passed into strassen multiplication!".to_string());
        }
        let object1 = args[0];
        let object2 = args[1];
    
        // Make sure it even exists in either.
        if !(is_matr(matr_ptr, object1) & is_matr(matr_ptr, object2)) {
            println!("Not enough matrices exist!");
            return Err("Not enough matrices exist!".to_string());
        }

        // Multiply matrix
       
        let matr_obj1 = matr_ptr.get(object1).unwrap();
        let matr_obj2 = matr_ptr.get(object2).unwrap();
        let (res, multi) = strassen_method(matr_obj1, matr_obj2);
        if res.is_err() {
            return Err(res.unwrap_err().to_string());
        }

        if vars_ptr.contains_key(result_multi_count_name) {
                vars_ptr.remove(result_multi_count_name);
        }
        vars_ptr.insert(result_multi_count_name.to_string(), multi as f64);
        matr_ptr.insert(result_matr_name.to_string(), res.unwrap());
        Ok(String::new())
        
    };

    let mut rmatr = |matr_ptr: &mut HashMap<String, Matrix<f64>>, vars_ptr: &mut HashMap<String, f64>| -> Result<String, String> {
        if codevec.len() < 2 {
            println!("Missing destination for random_matrix!");
            return Err("Missing destination for random_matrix!".to_string());
        }
        let result_matr_name = codevec[1];
        if args.len() < 2 {
            println!("Not enough arguments passed into random_matrix!");
            return Err("Not enough arguments passed into random_matrix!".to_string());
        }
        
        let object1 = args[0].parse::<usize>();
        let object2 = args[1].parse::<usize>();
        if !is_var(vars_ptr, args[0]) & object1.is_err()  {
            return Err(object1.unwrap_err().to_string());
        }
        if !is_var(vars_ptr, args[1]) & object2.is_err()  {
            return Err(object2.unwrap_err().to_string());
        }

        let mut dim1 = 0;
        if is_var(vars_ptr, args[0]) {
            dim1 = *vars_ptr.get(args[0]).unwrap() as usize;
        } else {
            dim1 = object1.unwrap();
        }
        let mut dim2 = 0;
        if is_var(vars_ptr, args[1]) {
            dim2 = *vars_ptr.get(args[1]).unwrap() as usize;
        } else {
            dim2 = object2.unwrap();
        }
        // Generate Matrix
        let random_matr: Matrix<f64> = random_matrix((dim1, dim2));
        matr_ptr.insert(result_matr_name.to_string(), random_matr);
        Ok(String::new())
        
    };

    if operation == "print" {
        return print(matr_ptr, vars_ptr);
    } else if operation == "naive_multiplication" {
        return nmulti(matr_ptr, vars_ptr);
    } else if operation == "strassen_multiplication" {
        return smulti(matr_ptr, vars_ptr);
    } else if operation == "random_matrix" {
        return rmatr(matr_ptr, vars_ptr);
    } else {
        Err("Call function does not exist".to_string())
    }

    // Find a way to handle operations
}
fn loop_op(codevec: Vec<&str>) {
    
}
fn export_op(codevec: Vec<&str>) {
    
}
fn import_op(codevec: Vec<&str>) {
    
}