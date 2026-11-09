// Author: Vincent Dellmar
// Created: 11-07-2026 (Happy Birthday to me!)
// Email: vide1819@colorado.edu
// Description:
/*
    This is a module for using creating programs using matrlang (a custom language by me) 
    that allows you to do matrix operations in a coded file. Allows exporting data into a csv.
*/
#![allow(unused_imports)]
use std::{any::{Any, TypeId, type_name}, collections::HashMap, env::{self, Args, vars}, fs::*, hash::Hash, io::{BufRead, BufReader, Read}, path::PathBuf, result, thread::{self, JoinHandle}, time::Duration};
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
pub fn run2(path: PathBuf) {
    /*
        commands:
        define VARIABLE_NAME/#VARIABLE VARIABLE/NUMBER/"VALUE"/(DIM0,DIM1) *[VALUES];
        set_flag FLAG_NAME; // set_flag makes a variable with a number set to the line number it is on.
        goto FLAG_NAME; // requires set_flag to use this.
        goto_ifgt (VARIABLE1/NUMBER1, VARIABLE2/NUMBER2) FLAG_NAME; 
        goto_iflt (VARIABLE1/NUMBER1, VARIABLE2/NUMBER2) FLAG_NAME;
        goto_ifeq (VARIABLE1/NUMBER1, VARIABLE2/NUMBER2) FLAG_NAME;
        op OPERATION_NAME (ARG1,...,ARGN) NAME;
        call FUNCTION_NAME (ARG1,...,ARGN) (RESULT1,...,RESULTN);
        export "PATH_VALUE"/VARIABLE SET/ADD VALUE;
     */
    
    // Important for logging output and errors.
    let mut log_buffer: String = String::new();
    let log = |result: Result<String, String>| {
        if result.is_err() {
            println!("{}", result.unwrap_err());
        }
    };

    // Opens File.
    let mut prgm_file = File::open(path).unwrap();

    // Reads file into a buffer string.
    let mut buffer = {
        let mut buffer_reader = BufReader::new(prgm_file);
        let mut buffer = vec![];

        buffer_reader.read_to_end(&mut buffer).unwrap();
        String::from_utf8_lossy(&buffer).trim().to_string()
    };

    // Produces a HashMap of Instructions (OpCode, (UNPROCESSED) Data).
    fn produce_raw_instruction_map(mut buffer: String) 
    -> Result<Vec<(String, String)>, String> 
    {
        if buffer.len() < 1 {return Err("No instructions to process, file is empty!".to_string())}

        let mut raw_instruction_map: Vec<(String, String)> = Vec::new();
        // Decomposes lines into opcodes & their data (unprocessed)
        while buffer.len() > 0 {
            // Opcode Separator. Used for Splitting.
            let mut separator_index = buffer.len();
            {
                let opcode_separator_index = buffer.find(' ');
                if opcode_separator_index.is_some() {
                    separator_index = opcode_separator_index.unwrap();
                }
            }

            // Opcode
            let mut opcode = buffer[0..separator_index].trim().to_string();
            
            // New buffer.
            if separator_index == buffer.len() {
                buffer = String::new();
            } else if separator_index > 0 {
                buffer = buffer[separator_index+1..].to_string();
            };

            // Comment handler.
            if opcode.contains("//") {
                // Find end of line.

                // Bug in here. Find after shower.
                let EOL = buffer.find('\n');
                let EOL2 = opcode.find('\n');
                
                if EOL.is_some() {
                    buffer = buffer[EOL.unwrap()+1..].to_string();
                } else if EOL2.is_some() {
                    let mut c = opcode.clone();
                    c = (&c[EOL2.unwrap()+1..]).to_string();
                    c.push_str(" ");
                    c.push_str(&buffer);
                    buffer = c;
                } else {
                    break;
                }
                continue;
            }

            // Find semicolon.
            if !buffer.contains(";") {return Err("Instruction missing end of sentence keyword ';'!".to_string())}
            let semi = buffer.find(';').unwrap();

            // Data scraper.
            let data = buffer[0..semi].trim().to_string();

            // New buffer.
            buffer = buffer[semi+1..].to_string();

            // Adds to hashmap.
            raw_instruction_map.push((opcode, data));
        }
        Ok(raw_instruction_map)
    }

    // Produces a Vector of Instructions (OpCode, (PROCESSED) Data...).
    fn process_raw_instruction_map(instruction_map: Vec<(String, String)>)
    -> Result<Vec<(String, Vec<String>)>, String>
    {
        // List of valid opcodes.
        let mut instruction_list: Vec<(String, Vec<String>)> = Vec::new();

        let split_spaces = |input: String| -> Vec<String> {
            let raw_output: Vec<&str> = input.split(' ').collect();
            let mut output: Vec<String> = Vec::new();
            for raw_output_val in raw_output.into_iter() {
                output.push(raw_output_val.to_string());
            }
            return output;
        };
        let split_commas = |input: String| -> Vec<String> {
            let raw_output: Vec<&str> = input.split(',').collect();
            let mut output: Vec<String> = Vec::new();
            for raw_output_val in raw_output.into_iter() {
                output.push(raw_output_val.trim().to_string());
            }
            return output;
        };
        let trim_parenthetical = |input: String| -> Result<String, String> {
            if !input.contains('(') | !input.contains(')') {return Err("Missing parentheses!".to_string());}
            let startpos = input.find('(').unwrap();
            let endpos = input.rfind(')').unwrap();
            if endpos < startpos {return Err("Parentheses in wrong order!".to_string());}
            return Ok((&input)[startpos+1..endpos].to_string());
        };
        let trim_brackets = |input: String| -> Result<String, String> {
            if !input.contains('[') | !input.contains(']') {return Err("Missing brackets!".to_string());}
            let startpos = input.find('[').unwrap();
            let endpos = input.rfind(']').unwrap();
            if endpos < startpos {return Err("Brackets in wrong order!".to_string());}
            return Ok((&input)[startpos..endpos].to_string());
        };

        let collect_args = |input: String| -> Result<Vec<String>, String> {
            let mut args = split_commas(input);
            for arg in args.iter_mut() {
                *arg = arg.trim().to_string();
            }
            return Ok(args);
        };

        let mut valid_opcodes: HashMap<String, Box<dyn Fn(usize, String) -> Result<Vec<String>, String>>> = HashMap::new();
        valid_opcodes.insert("define".to_string(), Box::new(|line_num: usize, unprocessed_data: String| -> Result<Vec<String>, String> {
            // Split spaces.
            let list = split_spaces(unprocessed_data);

            // There needs to be atleast 2 variables (3 if its a matrix).
            if list.len() < 2 {return Err("Not enough variables used for define!".to_string())};

            let mut args: Vec<String> = Vec::new();

            // Sets First Value Type.
            args.push(list[0].clone());

            // see if second arg is a variable, a string, a number, or a matrix.
            {
                let trim_paren_result = trim_parenthetical(list[1].clone());
                // Indicates matrix.
                if trim_paren_result.is_ok() {
                    let dims = split_commas(trim_paren_result.unwrap());
                    if dims.len() != 2 {return Err("Wrong number of dimensions used, dim has to be 2.".to_string());}
                    args.push(dims[0].clone());
                    args.push(dims[1].clone());
                    if list.len() > 3 {return Err("Only 1 array is necessary to be supplied for defining matrix!".to_string())}
                    else if list.len() != 3 {return Err("Missing array for defining matrix!".to_string())};
                    args.push(list[2].clone());
                } else {
                    args.push(list[1].clone());
                }
                
            };
            return Ok(args);
        }));
        valid_opcodes.insert("set_flag".to_string(), Box::new(|line_num: usize, unprocessed_data: String| -> Result<Vec<String>, String> {
            // Split spaces.
            let list = split_spaces(unprocessed_data);

            // There needs to be a variable.
            if list.len() < 1 {return Err("missing variable name for set_flag!".to_string())};

            // Uses the line index.
            return Ok(vec![line_num.to_string()]);
        }));
        valid_opcodes.insert("goto".to_string(), Box::new(|line_num: usize, unprocessed_data: String| -> Result<Vec<String>, String> {
            // Split spaces.
            let list = split_spaces(unprocessed_data);

            // There needs to be a variable.
            if list.len() < 1 {return Err("missing flag name for goto!".to_string())};
            return Ok(vec![list[0].clone()]);
        }));
        valid_opcodes.insert("goto_ifgt".to_string(), Box::new(|line_num: usize, unprocessed_data: String| -> Result<Vec<String>, String> {
            // Split spaces.
            let list = split_spaces(unprocessed_data);
            let mut args: Vec<String> = Vec::new();
            // There needs to be a variable.
            if list.len() != 2 {return Err("missing either condition or flag name, or both (ifgt)!".to_string())};
            
            // Checks if parentheses exists.
            let trim_paren_result = trim_parenthetical(list[0].clone());
            if trim_paren_result.is_err() {return Err(trim_paren_result.unwrap_err());}

            // Checks for conditions.
            let conds = split_commas(trim_paren_result.unwrap());
            if conds.len() != 2 {return Err("Conditional in goto_ifgt has to have 2 variables/numbers!".to_string());}
            args.push(conds[0].clone());
            args.push(conds[1].clone());
            args.push(list[1].clone());

            return Ok(args);
        }));
        valid_opcodes.insert("goto_iflt".to_string(), Box::new(|line_num: usize, unprocessed_data: String| -> Result<Vec<String>, String> {
            // Split spaces.
            let list = split_spaces(unprocessed_data);
            let mut args: Vec<String> = Vec::new();
            // There needs to be a variable.
            if list.len() != 2 {return Err("missing either condition or flag name, or both (iflt)!".to_string())};
            
            // Checks if parentheses exists.
            let trim_paren_result = trim_parenthetical(list[0].clone());
            if trim_paren_result.is_err() {return Err(trim_paren_result.unwrap_err());}

            // Checks for conditions.
            let conds = split_commas(trim_paren_result.unwrap());
            if conds.len() != 2 {return Err("Conditional in goto_iflt has to have 2 variables/numbers!".to_string());}
            args.push(conds[0].clone());
            args.push(conds[1].clone());
            args.push(list[1].clone());

            return Ok(args);
        }));
        valid_opcodes.insert("goto_ifeq".to_string(), Box::new(|line_num: usize, unprocessed_data: String| -> Result<Vec<String>, String> {
            // Split spaces.
            let list = split_spaces(unprocessed_data);
            let mut args: Vec<String> = Vec::new();
            // There needs to be a variable.
            if list.len() != 2 {return Err("missing either condition or flag name, or both (ifeq)!".to_string())};
            
            // Checks if parentheses exists.
            let trim_paren_result = trim_parenthetical(list[0].clone());
            if trim_paren_result.is_err() {return Err(trim_paren_result.unwrap_err());}

            // Checks for conditions.
            let conds = split_commas(trim_paren_result.unwrap());
            if conds.len() != 2 {return Err("Conditional in goto_ifeq has to have 2 variables/numbers!".to_string());}
            args.push(conds[0].clone());
            args.push(conds[1].clone());
            args.push(list[1].clone());
            

            return Ok(args);
        }));
        valid_opcodes.insert("op".to_string(), Box::new(|line_num: usize, unprocessed_data: String| -> Result<Vec<String>, String> {
            // Split spaces.
            let list = split_spaces(unprocessed_data);
            let mut args: Vec<String> = Vec::new();
            // There needs to be a variable.
            if list.len() != 3 {return Err("missing either the operation name, operation argument(s), or operation output!".to_string())};
            
            // Operation name.
            args.push(list[0].clone());

            // Checks if parentheses exists.
            let trim_paren_result = trim_parenthetical(list[1].clone());
            if trim_paren_result.is_err() {return Err(trim_paren_result.unwrap_err());}

            // Checks for parameters.
            let mut params = split_commas(trim_paren_result.unwrap());
            if params.len() < 1 {return Err("operations require atleast one parameter!".to_string());}
            args.append(&mut params);

            // Checks for output.
            args.push(list[2].clone());
            return Ok(args);
        }));
        valid_opcodes.insert("call".to_string(), Box::new(|line_num: usize, unprocessed_data: String| -> Result<Vec<String>, String> {
            // Split spaces.
            let list = split_spaces(unprocessed_data);
            let mut args: Vec<String> = Vec::new();
            // There needs to be a variable.
            if list.len() < 2 {return Err("missing either the call name, call argument(s), or call output(s)!".to_string())};
            
            // Call name.
            args.push(list[0].clone());

            // Parameters.
            {
                // Checks if parentheses exists.
                let trim_paren_result = trim_parenthetical(list[1].clone());
                if trim_paren_result.is_err() {return Err(trim_paren_result.unwrap_err());}

                // Checks for parameters.
                let mut params = split_commas(trim_paren_result.unwrap());
                if params.len() < 1 {return Err("operations require atleast one parameter!".to_string());}
                args.append(&mut params);
            }

            if list[2].len() > 3 {return Err("call has too many arguments!".to_string())};
            // Not all calls return values.
            if list[2].len() == 2 {return Ok(args)};

            // Outputs.
            let trim_paren_result = trim_parenthetical(list[2].clone());
            // Checks if parentheses exists.
            if trim_paren_result.is_ok() {
                // Checks for parameters.
                let mut params = split_commas(trim_paren_result.unwrap());
                if params.len() < 1 {return Err("parentheses provided, but no arguments passed!".to_string());}
                args.append(&mut params);
            }
            return Ok(args);
        }));
        // Need to make export work.
        vec!["define","set_flag","goto","goto_ifgt","goto_iflt","goto_ifeq","op","call","export"];
        let mut line = 0 as usize;
        
        for (opcode, unprocessed_data) in instruction_map.into_iter() {
            // Grabs the opcode processor.
            let opcode_processor_result = valid_opcodes.get(&opcode);
            if opcode_processor_result.is_none() {return Err("Invalid instruction used!".to_string());}
            let opcode_processor = opcode_processor_result.unwrap();
            // Processes the data using said opcode processor.
            let processed_data_result = opcode_processor(line, unprocessed_data);
            if processed_data_result.is_err() {return Err(processed_data_result.unwrap_err().to_string());}
            let processed_data = processed_data_result.unwrap();
            // Increment the line.
            instruction_list.push((opcode, processed_data));
            line += 1;
        }
        Ok(instruction_list)
    }
    
    // Produces & Processes Buffer into Instruction Map.
    let result_instruction_list: Result<Vec<(String, Vec<String>)>, String> = match produce_raw_instruction_map(buffer) {
        Ok(raw_instruction_map) => process_raw_instruction_map(raw_instruction_map),
        bad_result => {Err(bad_result.unwrap_err())}
    };

    // Logs the error and returns.
    if result_instruction_list.is_err() {log(Err(result_instruction_list.unwrap_err())); return;}
    // Extracts instruction list.
    let instruction_list: Vec<(String, Vec<String>)> = result_instruction_list.unwrap();
    println!("Completed interpretation!");
    // Where variables are stored.
    let mut var_map: HashMap<String, Box<dyn Any>> = HashMap::new();
    
    let mut curline: usize = 0;
    // Defined macros!
    let get_type = |var_map: &HashMap<String, Box<dyn Any + 'static>>, value: String| -> String {
        if value.starts_with("\"") & value.ends_with("\"") {
            return "string".to_string();
        } else if value.parse::<f64>().is_ok() {
            return "number".to_string();
        } else if value.starts_with("#") {
            return "pointer".to_string();
        } else if var_map.contains_key(&value) {
            return "variable".to_string();
        } else {
            return "variable".to_string();
        }
        // Add Support for Matrix Type.
    };
    let get_types = |var_map: &HashMap<String, Box<dyn Any + 'static>>, values: Vec<String>| -> Vec<String> {
        let mut types_array: Vec<String> = Vec::new();
        for value in values.iter() {
            types_array.push(get_type(var_map, value.clone()));
        }
        return types_array;
    };
    // Defined operation innerworks.
        let mut valid_instructions: HashMap<String, Box<dyn Fn(&mut HashMap<String, Box<dyn Any + 'static>>, Vec<String>) -> Result<String, String>>> = HashMap::new();
 
        valid_instructions.insert("define".to_string(), Box::new(|var_map: &mut HashMap<String, Box<dyn Any + 'static>>, fields: Vec<String>| -> Result<String, String> {
            let definition_type: String = get_type(var_map, fields[0].clone());
            #[allow(unused_assignments)]
            let mut definition_name: String = String::new();
            let mut definition_value: String = String::new();
            // Pointers with boxed = holy larp
            // Definition Variable
            if definition_type == "pointer" {
                let pointer_value = (&(fields[0].clone())[1..]).to_string();
                // Make sure pointer to value exists.
                if !var_map.contains_key(&pointer_value) {
                    return Err("attempt to use define with a name pointer to a non existent variable!".to_string());
                }
                let pointer_value_result = var_map.get(&pointer_value).unwrap();
                // I hate boxed.
                let downcast = (*pointer_value_result).downcast_ref::<String>();
                if downcast.is_none() {
                    return Err("attempt to use define with a name pointer to a variable of wrong primitive type!".to_string());
                }
                definition_name = downcast.unwrap().to_string();
            } else if definition_type == "variable" {
                definition_name = fields[0].clone();
            } else {
                return Err("Unsupported primitive type used for defining variable!".to_string());
            }
            let value_type: String = get_type(var_map, fields[1].clone());
            // Definition Value
            if value_type == "variable" {
                let pointer_value = fields[1].clone();
                // Make sure pointer to value exists.
                if !var_map.contains_key(&pointer_value) {
                    return Err("attempt to use define with a value pointer to a non existent variable!".to_string());
                }
                let pointer_value_result = var_map.get(&pointer_value).unwrap();
                // I hate boxed.
                var_map.insert(definition_name.clone(), Box::new((*pointer_value_result).downcast_ref::<&String>().unwrap().to_string()));
            } else if value_type == "string" {
                let value = fields[1].clone();
                var_map.insert(definition_name.clone(), Box::new(fields[1].clone()));

            } else if value_type == "number" {
                var_map.insert(definition_name.clone(), Box::new(fields[1].parse::<f64>().unwrap()));
            } else {
                return Err("Unsupported primitive type used for defining variable!".to_string());
            }
            // Add Support for Matrix Type.
            Ok("".to_string())
        }));


    loop {
        if curline > instruction_list.len() - 1 {break};
        let (current_opcode, current_instruction) = instruction_list.get(curline).unwrap();
        println!("current opcode: {} -> {:?}", current_opcode, current_instruction);
        curline += 1;
        // Fetch Instruction.
        let instruction_result = valid_instructions.get(current_opcode);
        
        // Mandatory for safety.
        if instruction_result.is_none() {
            log(Err("Non existent instruction!".to_string()));
            break;
        }

        // Execute Instruction.
        let executed_result = instruction_result.unwrap()(&mut var_map, current_instruction.clone());

        // Log output.
        log(executed_result);
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