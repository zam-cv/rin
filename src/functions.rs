use crate::parser::Rule;
use anyhow::{anyhow, Result};

pub fn returns_value(
    value_pair: pest::iterators::Pair<Rule>,
    name: &String,
    output: &mut String,
) -> Result<()> {
    match value_pair.as_str() {
        "input()" => {
            output.push_str(&format!("INPUT\nSTORE {}\nCLEAR\n", name));
        }
        _ => return Err(anyhow!("Unsupported function: {}", value_pair.as_str())),
    };

    Ok(())
}
