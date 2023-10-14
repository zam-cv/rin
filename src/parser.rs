use anyhow::{anyhow, Result};
use pest::Parser;
use std::collections::HashMap;

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

const DEFAULT_VALUE: i16 = 0;

pub fn generate(code: &str) -> Result<String> {
    let mut variables: HashMap<String, (String, i16)> = HashMap::new();
    let mut output = String::new();

    let parsed = MyParser::parse(Rule::program, &code).unwrap_or_else(|e| {
        panic!("Parsing error: {}", e);
    });

    for pair in parsed {
        match pair.as_rule() {
            Rule::statement => {
                let mut inner = pair.into_inner();

                let ident = inner
                    .next()
                    .ok_or_else(|| anyhow!("Missing identifier"))?
                    .as_str()
                    .to_string();

                let type_value = inner
                    .next()
                    .ok_or_else(|| anyhow!("Missing type"))?
                    .as_str()
                    .to_string();

                let value_pair = inner.next().ok_or_else(|| anyhow!("Missing value"))?;

                match value_pair.as_rule() {
                    Rule::number => {
                        let value = value_pair.as_str().parse::<i16>()?;
                        variables.insert(ident, (type_value, value));
                    }
                    Rule::function_call => match value_pair.as_str() {
                        "input()" => {
                            output.push_str(&format!("INPUT\nSTORE {}\nCLEAR\n", &ident));
                            variables.insert(ident, (type_value, DEFAULT_VALUE));
                        }
                        _ => return Err(anyhow!("Unsupported function: {}", value_pair.as_str())),
                    },
                    _ => return Err(anyhow!("Invalid value assignment")),
                }
            }
            Rule::function_call => {
                let mut inner = pair.into_inner();

                let ident = inner
                    .next()
                    .ok_or_else(|| anyhow!("Missing identifier"))?
                    .as_str()
                    .to_string();

                let params = inner
                    .next()
                    .ok_or_else(|| anyhow!("Missing parameters"))?
                    .as_str()
                    .to_string();
                
                match ident.as_str() {
                    "print" => {
                        output.push_str(&format!("\nLOAD {}\nOUTPUT\n", params));
                    }
                    _ => return Err(anyhow!("Unsupported function: {}", ident)),
                }
            },
            _ => {}
        }
    }

    output.push_str("\nHALT\n\n");

    for (ident, (stype, value)) in variables {
        output.push_str(&format!("{},   {} {}\n", ident, stype, value));
    }

    output.pop();
    Ok(output)
}