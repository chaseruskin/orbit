//
//  Copyright (C) 2022-2024  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

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
