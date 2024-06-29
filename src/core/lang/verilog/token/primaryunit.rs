use crate::util::anyerror::CodeFault;
use std::collections::HashMap;

pub fn collect_units(files: &Vec<String>) -> Result<HashMap<String, String>, CodeFault> {
    let result: HashMap<String, String> = HashMap::new();
    // iterate through all source files
    for source_file in files {
        // only read the HDL files
        if crate::core::fileset::is_verilog(&source_file) == true {
            println!("parse verilog: {:?}", source_file);
            // parse text into Verilog symbols
            // println!("Detected verilog file: {}", source_file);
            let _contents = std::fs::read_to_string(&source_file).unwrap();
            // let symbols = VHDLParser::read(&contents).into_symbols();
        }
    }
    Ok(result)
}
