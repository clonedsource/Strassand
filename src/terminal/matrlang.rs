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
pub fn run(path: PathBuf) {
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
            return Ok((&input)[startpos+1..endpos].to_string());
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
            return Ok(vec![list[0].clone(), (line_num + 1).to_string()]);
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
            args.push(conds[0].trim().to_string());
            args.push(conds[1].trim().to_string());
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
            args.push(conds[0].trim().to_string());
            args.push(conds[1].trim().to_string());
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
            args.push(conds[0].trim().to_string());
            args.push(conds[1].trim().to_string());
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
            for param in params.iter_mut() {
                *param = param.trim().to_string();
            }
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
                for param in params.iter_mut() {
                    *param = param.trim().to_string();
                }
                if params.len() < 1 {return Err("operations require atleast one parameter!".to_string());}
                args.append(&mut params);
            }

            if list.len() > 3 {return Err("call has too many arguments!".to_string())};
            // Not all calls return values.
            if list.len() == 2 {return Ok(args)};

            // Outputs.
            let trim_paren_result = trim_parenthetical(list[2].clone());
            // Checks if parentheses exists.
            if trim_paren_result.is_ok() {
                // Checks for parameters.
                let mut params = split_commas(trim_paren_result.unwrap());
                for param in params.iter_mut() {
                    *param = param.trim().to_string();
                }
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
    
    // Useful helpers.
    let trim_brackets = |input: String| -> Result<String, String> {
        if !input.contains('[') | !input.contains(']') {return Err("Missing brackets!".to_string());}
        let startpos = input.find('[').unwrap();
        let endpos = input.rfind(']').unwrap();
        if endpos < startpos {return Err("Brackets in wrong order!".to_string());}
        return Ok((&input)[startpos+1..endpos].to_string());
    };
    let split_commas = |input: String| -> Vec<String> {
        let raw_output: Vec<&str> = input.split(',').collect();
        let mut output: Vec<String> = Vec::new();
        for raw_output_val in raw_output.into_iter() {
            output.push(raw_output_val.trim().to_string());
        }
        return output;
    };
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
            if let Some(res_value) = var_map.get(&value) {
                if let Some(matrix) = res_value.downcast_ref::<Matrix<f64>>() {
                    return "matrix".to_string();
                } else if let Some(number) = res_value.downcast_ref::<f64>() {
                    return "number".to_string();
                } else if let Some(string) = res_value.downcast_ref::<String>() {
                    return "string".to_string();
                }
            }
            return "variable".to_string();
        } else if value.starts_with("[") {
            return "matrix".to_string();
        } else {
            return "variable".to_string();
        }
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
            } else if var_map.contains_key(&fields[0].clone()) { // variable.
                definition_name = fields[0].clone();
            } else {
                return Err("Unsupported primitive type used for defining variable!".to_string());
            }
            // matrix name, dim0, dim1, vector (unparsed)
            if fields.len() == 4 {
                let (dim0_parse, dim1_parse) = (fields[1].parse::<usize>(), fields[2].parse::<usize>());
                // Handles parsing errors.
                if let Err(error) = dim0_parse {return Err(error.to_string())}
                else if let Err(error) = dim1_parse {return Err(error.to_string())}
                // Defines the dimension.
                let dim = (dim0_parse.unwrap(), dim1_parse.unwrap());
                // Parses vector.
                let mut data: Vec<f64> = Vec::new();
                let data_string_trimmed = trim_brackets(fields[3].clone());
                if let Err(error) = data_string_trimmed {
                    return Err(error);
                }
                let data_vec = split_commas(data_string_trimmed.unwrap());
                for vec_val in data_vec.into_iter() {
                    let parsed_val = vec_val.parse::<f64>();
                    if let Err(error) = parsed_val {return Err(error.to_string());}
                    else if let Ok(value) = parsed_val {data.push(value);}
                }

                // Matrix creation.
                let matrix: Matrix<f64> = Matrix::new(dim, Some(data));
                var_map.insert(definition_name, Box::new(matrix));
                return Ok(String::new());                
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
            Ok("".to_string())
        }));
        valid_instructions.insert("set_flag".to_string(), Box::new(|var_map: &mut HashMap<String, Box<dyn Any + 'static>>, fields: Vec<String>| -> Result<String, String> {
            
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
            
            if let Ok(value) = fields[1].parse::<usize>() {
                var_map.insert(definition_name.clone(), Box::new(value));

            } else {
                return Err("Attempted to set flag with non usize number!".to_string());
            }
            // Add Support for Matrix Type.
            Ok("".to_string())
        }));
        valid_instructions.insert("goto".to_string(), Box::new(|var_map: &mut HashMap<String, Box<dyn Any + 'static>>, fields: Vec<String>| -> Result<String, String> {
            // BUG, GOES TO LINE BEFORE THIS ONE. WHAT THE FUCK?
            if get_type(var_map, fields[0].clone()) == "variable" {
                let pointer_value = fields[0].clone();
                // Make sure pointer to value exists.
                if !var_map.contains_key(&pointer_value) {
                    return Err("attempt to use goto with a value pointer to a non existent variable!".to_string());
                }
                let pointer_value_result = var_map.get(&pointer_value).unwrap();
                // I hate boxed.
                let value = (*pointer_value_result).downcast_ref::<usize>();
                if let Some(nvalue) = value {
                    return Ok(nvalue.to_string());
                }
            } else if let Ok(value) = fields[0].parse::<usize>() {
                return Ok(value.to_string());
            }
            return Err("Undeclared flag used in goto!".to_string());
        }));
        valid_instructions.insert("goto_ifgt".to_string(), Box::new(|var_map: &mut HashMap<String, Box<dyn Any + 'static>>, fields: Vec<String>| -> Result<String, String> {
            #[allow(unused)]
            let mut cond1: f64 = 0_f64;
            #[allow(unused)]
            let mut cond2: f64 = 0_f64;
            let cond1_type: String = get_type(var_map, fields[0].clone());
            // Cond1
            if var_map.contains_key(&fields[0]) {
                let pointer_value = fields[0].clone();
                // Make sure pointer to value exists.
                if !var_map.contains_key(&pointer_value) {
                    return Err("attempt to define condition 1 with a non existent variable!".to_string());
                }
                let pointer_value_result = var_map.get(&pointer_value).unwrap();
                // I hate boxed.
                let downcast_result = (*pointer_value_result).downcast_ref::<f64>();
                if let Some(downcast) = downcast_result {
                    cond1 = downcast.clone();
                } else {
                    return Err("Non numerical variable used for condition 1 in goto!".to_string());
                }
            } else if cond1_type == "number" {
                let value_parsed = fields[0].parse::<f64>();
                if value_parsed.is_err() {
                    return Err(value_parsed.clone().unwrap_err().to_string());
                }
                cond1 = value_parsed.unwrap();
            } else {
                return Err("Unsupported primitive type used for condition in goto!".to_string());
            }
            let cond2_type: String = get_type(var_map, fields[1].clone());
            // Cond2
            if var_map.contains_key(&fields[1]) {
                let pointer_value = fields[1].clone();
                // Make sure pointer to value exists.
                if !var_map.contains_key(&pointer_value) {
                    return Err("attempt to define condition 2 with a non existent variable!".to_string());
                }
                let pointer_value_result = var_map.get(&pointer_value).unwrap();
                // I hate boxed.
                let downcast_result = (*pointer_value_result).downcast_ref::<f64>();
                if let Some(downcast) = downcast_result {
                    cond2 = downcast.clone();
                } else {
                    return Err("Non numerical variable used for condition 2 in goto!".to_string());
                }
            } else if cond2_type == "number" {
                let value_parsed = fields[1].parse::<f64>();
                if value_parsed.is_err() {
                    return Err(value_parsed.clone().unwrap_err().to_string());
                }
                cond2 = value_parsed.unwrap();
            } else {
                return Err("Unsupported primitive type used for condition in goto!".to_string());
            }
            if get_type(var_map, fields[2].clone()) == "variable" {
                let pointer_value = fields[2].clone();
                // Make sure pointer to value exists.
                if !var_map.contains_key(&pointer_value) {
                    return Err("attempt to use goto with a value pointer to a non existent variable!".to_string());
                }
                let pointer_value_result = var_map.get(&pointer_value).unwrap();
                // I hate boxed.
                let value = (*pointer_value_result).downcast_ref::<usize>().unwrap().to_string();
                if let Ok(nvalue) = value.parse::<usize>() {
                    if cond1 <= cond2 {
                        return Ok("S".to_string());
                    }
                    return Ok(nvalue.to_string());
                }
            } else if let Ok(value) = fields[2].parse::<usize>() {
                if cond1 <= cond2 {
                    return Ok("S".to_string());
                }
                return Ok(value.to_string());
            }
            return Err("Undeclared flag used in goto!".to_string());
        }));
        valid_instructions.insert("goto_ifeq".to_string(), Box::new(|var_map: &mut HashMap<String, Box<dyn Any + 'static>>, fields: Vec<String>| -> Result<String, String> {
            #[allow(unused)]
            let mut cond1: f64 = 0_f64;
            #[allow(unused)]
            let mut cond2: f64 = 0_f64;
            let cond1_type: String = get_type(var_map, fields[0].clone());
            // Cond1
            if var_map.contains_key(&fields[0]) {
                let pointer_value = fields[0].clone();
                // Make sure pointer to value exists.
                if !var_map.contains_key(&pointer_value) {
                    return Err("attempt to define condition 1 with a non existent variable!".to_string());
                }
                let pointer_value_result = var_map.get(&pointer_value).unwrap();
                // I hate boxed.
                let downcast_result = (*pointer_value_result).downcast_ref::<f64>();
                if let Some(downcast) = downcast_result {
                    cond1 = downcast.clone();
                } else {
                    return Err("Non numerical variable used for condition 1 in goto!".to_string());
                }
            } else if cond1_type == "number" {
                let value_parsed = fields[0].parse::<f64>();
                if value_parsed.is_err() {
                    return Err(value_parsed.clone().unwrap_err().to_string());
                }
                cond1 = value_parsed.unwrap();
            } else {
                return Err("Unsupported primitive type used for condition in goto!".to_string());
            }
            let cond2_type: String = get_type(var_map, fields[1].clone());
            // Cond2
            if var_map.contains_key(&fields[1]) {
                let pointer_value = fields[1].clone();
                // Make sure pointer to value exists.
                if !var_map.contains_key(&pointer_value) {
                    return Err("attempt to define condition 2 with a non existent variable!".to_string());
                }
                let pointer_value_result = var_map.get(&pointer_value).unwrap();
                // I hate boxed.
                let downcast_result = (*pointer_value_result).downcast_ref::<f64>();
                if let Some(downcast) = downcast_result {
                    cond2 = downcast.clone();
                } else {
                    return Err("Non numerical variable used for condition 2 in goto!".to_string());
                }
            } else if cond2_type == "number" {
                let value_parsed = fields[1].parse::<f64>();
                if value_parsed.is_err() {
                    return Err(value_parsed.clone().unwrap_err().to_string());
                }
                cond2 = value_parsed.unwrap();
            } else {
                return Err("Unsupported primitive type used for condition in goto!".to_string());
            }
            if get_type(var_map, fields[2].clone()) == "variable" {
                let pointer_value = fields[2].clone();
                // Make sure pointer to value exists.
                if !var_map.contains_key(&pointer_value) {
                    return Err("attempt to use goto with a value pointer to a non existent variable!".to_string());
                }
                let pointer_value_result = var_map.get(&pointer_value).unwrap();
                // I hate boxed.
                let value = (*pointer_value_result).downcast_ref::<usize>().unwrap().to_string();
                if let Ok(nvalue) = value.parse::<usize>() {
                    if cond1 != cond2 {
                        return Ok("S".to_string());
                    }
                    return Ok(nvalue.to_string());
                }
            } else if let Ok(value) = fields[2].parse::<usize>() {
                if cond1 != cond2 {
                    return Ok("S".to_string());
                }
                return Ok(value.to_string());
            }
            return Err("Undeclared flag used in goto!".to_string());
        }));
        valid_instructions.insert("goto_iflt".to_string(), Box::new(|var_map: &mut HashMap<String, Box<dyn Any + 'static>>, fields: Vec<String>| -> Result<String, String> {
            #[allow(unused)]
            let mut cond1: f64 = 0_f64;
            #[allow(unused)]
            let mut cond2: f64 = 0_f64;
            let cond1_type: String = get_type(var_map, fields[0].clone());
            // Cond1
            if var_map.contains_key(&fields[0]) {
                let pointer_value = fields[0].clone();
                // Make sure pointer to value exists.
                if !var_map.contains_key(&pointer_value) {
                    return Err("attempt to define condition 1 with a non existent variable!".to_string());
                }
                let pointer_value_result = var_map.get(&pointer_value).unwrap();
                // I hate boxed.
                let downcast_result = (*pointer_value_result).downcast_ref::<f64>();
                if let Some(downcast) = downcast_result {
                    cond1 = downcast.clone();
                } else {
                    return Err("Non numerical variable used for condition 1 in goto!".to_string());
                }
            } else if cond1_type == "number" {
                let value_parsed = fields[0].parse::<f64>();
                if value_parsed.is_err() {
                    return Err(value_parsed.clone().unwrap_err().to_string());
                }
                cond1 = value_parsed.unwrap();
            } else {
                return Err("Unsupported primitive type used for condition in goto!".to_string());
            }
            let cond2_type: String = get_type(var_map, fields[1].clone());
            // Cond2
            if var_map.contains_key(&fields[1]) {
                let pointer_value = fields[1].clone();
                // Make sure pointer to value exists.
                if !var_map.contains_key(&pointer_value) {
                    return Err("attempt to define condition 2 with a non existent variable!".to_string());
                }
                let pointer_value_result = var_map.get(&pointer_value).unwrap();
                // I hate boxed.
                let downcast_result = (*pointer_value_result).downcast_ref::<f64>();
                if let Some(downcast) = downcast_result {
                    cond2 = downcast.clone();
                } else {
                    return Err("Non numerical variable used for condition 2 in goto!".to_string());
                }
            } else if cond2_type == "number" {
                let value_parsed = fields[1].parse::<f64>();
                if value_parsed.is_err() {
                    return Err(value_parsed.clone().unwrap_err().to_string());
                }
                cond2 = value_parsed.unwrap();
            } else {
                return Err("Unsupported primitive type used for condition in goto!".to_string());
            }
            if get_type(var_map, fields[2].clone()) == "variable" {
                let pointer_value = fields[2].clone();
                // Make sure pointer to value exists.
                if !var_map.contains_key(&pointer_value) {
                    return Err("attempt to use goto with a value pointer to a non existent variable!".to_string());
                }
                let pointer_value_result = var_map.get(&pointer_value).unwrap();
                // I hate boxed.
                let value = (*pointer_value_result).downcast_ref::<usize>().unwrap().to_string();
                if let Ok(nvalue) = value.parse::<usize>() {
                    if cond1 >= cond2 {
                        return Ok("S".to_string());
                    }
                    return Ok(nvalue.to_string());
                }
            } else if let Ok(value) = fields[2].parse::<usize>() {
                if cond1 >= cond2 {
                    return Ok("S".to_string());
                }
                return Ok(value.to_string());
            }
            return Err("Undeclared flag used in goto!".to_string());
        }));
        valid_instructions.insert("op".to_string(), Box::new(|var_map: &mut HashMap<String, Box<dyn Any + 'static>>, fields: Vec<String>| -> Result<String, String> {
            let operation_name: String = fields[0].clone();
            // Handle Operation name here.
            if !vec!["add","subtract","multiply","divide"].contains(&&operation_name.as_str()) {
                return Err(format!("Unknown operation call of '{}'!", operation_name));
            }
            let output_dest_raw: String = fields[fields.len() - 1].clone();
            if let Ok(bad_choice) = output_dest_raw.parse::<f64>() {
                return Err("Cannot use number as destination in operation!".to_string());
            }

            let output_value: Result<Box<dyn Any>, String> = if operation_name == "add" {
                if fields.len() != 4 {return Err("args of op 'add' need to be of length 2!".to_string())};
                let field1_type = get_type(&var_map, fields[1].clone());
                let field2_type = get_type(&var_map, fields[2].clone());
                // Make sure types are the same
                if field1_type != field2_type {
                    return Err("Cannot add values of different type together!".to_string())
                };
                let num_parse_1 = fields[1].parse::<f64>();
                let string_parse_1 = fields[1].parse::<String>();
                let var1: Box<&dyn Any> = if var_map.contains_key(&fields[1]) {
                    let var1 = var_map.get(&fields[1]).unwrap().as_ref();
                    Box::new(var1)
                } else if let Ok(num) = &num_parse_1 {
                    Box::new(num)
                } else if let Ok(str) = &string_parse_1 {
                    Box::new(str)
                } else {
                    return Err("Cannot parse variable 1 in op!".to_string());
                };
                let num_parse_2 = fields[2].parse::<f64>();
                let string_parse_2 = fields[2].parse::<String>();
                let var2: Box<&dyn Any> = if var_map.contains_key(&fields[1]) {
                    let var2 = var_map.get(&fields[1]).unwrap().as_ref();
                    Box::new(var2)
                } else if let Ok(num) = &num_parse_2 {
                    Box::new(num)
                } else if let Ok(str) = &string_parse_2 {
                    Box::new(str)
                } else {
                    return Err("Cannot parse variable 2 in op!".to_string());
                };
                
                let res: Box<dyn Any> = if field1_type == "string" {
                    Box::new(var1.downcast_ref::<String>().unwrap().clone().push_str(var2.downcast_ref::<String>().unwrap()))
                } else if field1_type == "number" {
                    Box::new(var1.downcast_ref::<f64>().unwrap() + (var2.downcast_ref::<f64>().unwrap()))
                } else if field1_type == "matrix" {
                    Box::new(var1.downcast_ref::<Matrix<f64>>().unwrap() + (var2.downcast_ref::<Matrix<f64>>().unwrap()))
                } else {
                    return Err("Could not perform operation!".to_string());
                };
                Ok(res)
            } else if operation_name == "subtract" {
                if fields.len() != 4 {return Err("args of op 'subtract' need to be of length 2!".to_string())};
                let field1_type = get_type(&var_map, fields[1].clone());
                let field2_type = get_type(&var_map, fields[2].clone());
                // Make sure types are the same
                if field1_type != field2_type {
                    return Err("Cannot add values of different type together!".to_string())
                };
                let num_parse_1 = fields[1].parse::<f64>();
                let var1: Box<&dyn Any> = if var_map.contains_key(&fields[1]) {
                    let var1 = var_map.get(&fields[1]).unwrap().as_ref();
                    Box::new(var1)
                } else if let Ok(num) = &num_parse_1 {
                    Box::new(num)
                } else {
                    return Err("Cannot parse variable 1 in op!".to_string());
                };
                let num_parse_2 = fields[2].parse::<f64>();
                let var2: Box<&dyn Any> = if var_map.contains_key(&fields[1]) {
                    let var2 = var_map.get(&fields[1]).unwrap().as_ref();
                    Box::new(var2)
                } else if let Ok(num) = &num_parse_2 {
                    Box::new(num)
                } else {
                    return Err("Cannot parse variable 2 in op!".to_string());
                };
                
                let res: Box<dyn Any> = if field1_type == "number" {
                    Box::new(var1.downcast_ref::<f64>().unwrap() - (var2.downcast_ref::<f64>().unwrap()))
                } else if field1_type == "matrix" {
                    Box::new(var1.downcast_ref::<Matrix<f64>>().unwrap() - (var2.downcast_ref::<Matrix<f64>>().unwrap()))
                } else {
                    return Err("Could not perform operation!".to_string());
                };
                Ok(res)
            } else if operation_name == "multiply" {
                if fields.len() != 4 {return Err("args of op 'multiply' need to be of length 2!".to_string())};
                let field1_type = get_type(&var_map, fields[1].clone());
                let field2_type = get_type(&var_map, fields[2].clone());
                // Make sure types are the same
                if field1_type == "string" || field2_type == "string" {
                    return Err("Cannot multiply anything by a string!".to_string())
                };
                let num_parse_1 = fields[1].parse::<f64>();
                let var1: Box<&dyn Any> = if var_map.contains_key(&fields[1]) {
                    let var1 = var_map.get(&fields[1]).unwrap().as_ref();
                    Box::new(var1)
                } else if let Ok(num) = &num_parse_1 {
                    Box::new(num)
                } else {
                    return Err("Cannot parse variable 1 in op!".to_string());
                };
                let num_parse_2 = fields[2].parse::<f64>();
                let var2: Box<&dyn Any> = if var_map.contains_key(&fields[1]) {
                    let var2 = var_map.get(&fields[1]).unwrap().as_ref();
                    Box::new(var2)
                } else if let Ok(num) = &num_parse_2 {
                    Box::new(num)
                } else {
                    return Err("Cannot parse variable 2 in op!".to_string());
                };
                
                let res: Box<dyn Any> = if field1_type == "number" {
                    Box::new(var1.downcast_ref::<f64>().unwrap() * (var2.downcast_ref::<f64>().unwrap()))
                } else if field1_type == "matrix" && field2_type == "number" {
                    Box::new(var1.downcast_ref::<Matrix<f64>>().unwrap() * (var2.downcast_ref::<f64>().unwrap().clone()))
                } else if field1_type == "number" && field2_type == "matrix" {
                    Box::new(var2.downcast_ref::<Matrix<f64>>().unwrap() * (var1.downcast_ref::<f64>().unwrap().clone()))
                } else {
                    return Err("Could not perform operation!".to_string());
                };
                Ok(res)
            } else if operation_name == "multiply" {
                if fields.len() != 4 {return Err("args of op 'divide' need to be of length 2!".to_string())};
                let field1_type = get_type(&var_map, fields[1].clone());
                let field2_type = get_type(&var_map, fields[2].clone());
                // Make sure types are the same
                if field1_type == "string" || field2_type == "string" {
                    return Err("Cannot divide anything by a string!".to_string())
                };
                let num_parse_1 = fields[1].parse::<f64>();
                let var1: Box<&dyn Any> = if var_map.contains_key(&fields[1]) {
                    let var1 = var_map.get(&fields[1]).unwrap().as_ref();
                    Box::new(var1)
                } else if let Ok(num) = &num_parse_1 {
                    Box::new(num)
                } else {
                    return Err("Cannot parse variable 1 in op!".to_string());
                };
                let num_parse_2 = fields[2].parse::<f64>();
                let var2: Box<&dyn Any> = if var_map.contains_key(&fields[1]) {
                    let var2 = var_map.get(&fields[1]).unwrap().as_ref();
                    Box::new(var2)
                } else if let Ok(num) = &num_parse_2 {
                    Box::new(num)
                } else {
                    return Err("Cannot parse variable 2 in op!".to_string());
                };
                
                let res: Box<dyn Any> = if field1_type == "number" {
                    Box::new(var1.downcast_ref::<f64>().unwrap() / (var2.downcast_ref::<f64>().unwrap()))
                } else if field1_type == "matrix" && field2_type == "number" {
                    Box::new(var1.downcast_ref::<Matrix<f64>>().unwrap() * (1.0_f64 / (var2.downcast_ref::<f64>().unwrap().clone())))
                } else if field1_type == "number" && field2_type == "matrix" {
                    Box::new(var2.downcast_ref::<Matrix<f64>>().unwrap() * (1.0_f64 / (var1.downcast_ref::<f64>().unwrap().clone())))
                } else {
                    return Err("Could not perform operation!".to_string());
                };
                Ok(res)
            } else {
                Err(format!("Unknown operation call of '{}'!", operation_name))
            };
            if output_value.is_err() {
                return Err(output_value.unwrap_err());
            }
            // Inserts value into map.
            var_map.insert(output_dest_raw, output_value.unwrap());

            return Ok(String::new());
        }));
        valid_instructions.insert("call".to_string(), Box::new(|var_map: &mut HashMap<String, Box<dyn Any + 'static>>, fields: Vec<String>| -> Result<String, String> {
            let call_name: String = fields[0].clone();
            // Handle Call name here.
            if !vec!["print","naive_multiplication","strassen_multiplication"].contains(&&call_name.as_str()) {
                return Err(format!("Unknown operation call of '{}'!", call_name));
            }

            let output_values: Result<Vec<Box<dyn Any>>, String> = if call_name == "print" {
                if fields.len() != 2 {return Err("args of call 'print' need to be of length 1!".to_string())};
                let field1_type = get_type(&var_map, fields[1].clone());

                let num_parse_1 = fields[1].parse::<f64>();
                let string_parse_1 = fields[1].parse::<String>();
                let var1: Box<&dyn Any> = if var_map.contains_key(&fields[1]) {
                    let var1 = var_map.get(&fields[1]).unwrap().as_ref();
                    Box::new(var1)
                } else if let Ok(num) = &num_parse_1 {
                    Box::new(num)
                } else if let Ok(str) = &string_parse_1 {
                    Box::new(str)
                } else {
                    return Err("Cannot parse variable 1 in call!".to_string());
                };
                
                let res: Box<dyn Any> = if field1_type == "string" {
                    let string = var1.downcast_ref::<String>().unwrap().clone();
                    println!("{string}");
                    Box::new(string)
                } else if field1_type == "number" {
                    let string = var1.downcast_ref::<f64>().unwrap().clone().to_string();
                    println!("{string}");
                    Box::new(string)
                } else if field1_type == "matrix" {
                    let string = var1.downcast_ref::<Matrix<f64>>().unwrap().matrix_str();
                    println!("{string}");
                    Box::new(string)
                } else {
                    return Err("Could not perform operation!".to_string());
                };
                Ok(vec![res])
            } else {
                Err(format!("Unknown call of '{}'!", call_name))
            };
            if output_values.is_err() {
                return Err(output_values.unwrap_err());
            }

            // calls typically log values.
            let mut OkString: String = String::new();

            // Sine this is a call we need to iterate through.
            for (index, output_value) in output_values.unwrap().into_iter().enumerate() {
                // Rework this to support multiple returns.
                let output_dest_raw: String = fields[fields.len() - 1 - index].clone();
                if let Ok(bad_choice) = output_dest_raw.parse::<f64>() {
                    return Err("Cannot use number as destination in call!".to_string());
                }
                // Do borrowing, create OkString.
                if let Some(borrow) = output_value.downcast_ref::<String>() {
                    OkString.push_str("\"");
                    OkString.push_str(borrow);
                    OkString.push_str("\"");
                } else if let Some(borrow) = output_value.downcast_ref::<f64>() {
                    OkString.push_str(borrow.clone().to_string().as_str());
                } else if let Some(borrow) = output_value.downcast_ref::<usize>() {
                    OkString.push_str(borrow.clone().to_string().as_str());
                } 
                // Inserts value into map.
                var_map.insert(output_dest_raw, output_value);
            }

            return Ok(OkString);
        }));
    // Set flags.
    for (current_opcode, current_instruction) in instruction_list.iter() {
        if current_opcode != "set_flag" {continue;}
        // Fetch Instruction.
        let instruction_result = valid_instructions.get(current_opcode);
        
        // Mandatory for safety.
        if instruction_result.is_none() {
            log(Err("Non existent instruction!".to_string()));
            break;
        }
        println!("Flag set!");
        // Execute Instruction.
        let executed_result = instruction_result.unwrap()(&mut var_map, current_instruction.clone());

        // Log output.
        log(executed_result);
    }

    loop {
        if curline > instruction_list.len() - 1 {break};
        let (current_opcode, current_instruction) = instruction_list.get(curline).unwrap();
        println!("current opcode: {} -> {:?}", current_opcode, current_instruction);
        curline += 1;
        // We did a previous loop doing this lol.
        if current_opcode == "set_flag" {continue;}
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
        log(executed_result.clone());
        // Only for goto, goto_ifgt, goto_iflt, goto_ifeq instructions.
        if current_opcode.contains("goto") & executed_result.is_ok() {
            let result = executed_result.unwrap();
            if result == "S" {continue;}

            // BUG IN GOTO THAT CAUSES THIS TO GO TO LINE BEFORE FLAG.
            curline = result.parse::<usize>().unwrap() - 1;
        }
    }
}

pub fn view_log(path: PathBuf)  {

}