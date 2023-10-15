use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use crate::parser::{
  resolve_instructions,
  Rule
};

pub fn while_loop(
    pair: pest::iterators::Pair<Rule>,
    variables: &mut BTreeMap<String, (String, u16)>,
    output: &mut String,
    count_mul: &mut u64,
    count_div: &mut u64,
) -> Result<()> {
    let mut inner = pair.into_inner();

    let var_a = inner
        .next()
        .ok_or_else(|| anyhow!("Missing valueA"))?
        .as_str();

    let condition = inner
        .next()
        .ok_or_else(|| anyhow!("Missing condition"))?
        .as_str()
        .to_string();

    let var_b = inner
        .next()
        .ok_or_else(|| anyhow!("Missing valueB"))?
        .as_str();

    let mut block_output = String::new();

    resolve_instructions(inner, variables, &mut block_output, count_mul, count_div)?;

    let tab = block_output
        .lines()
        .map(|line| format!("\t{}", line))
        .collect::<Vec<String>>()
        .join("\n");

    let operator = match condition.as_str() {
        "<" => "000",
        "==" => "400",
        ">" => "800",
        _ => return Err(anyhow!("Invalid condition")),
    };

    output.push_str(&format!(
        "\nWHILE, 	LOAD {var_a}		
    SUBT {var_b}
    SKIPCOND {operator}
    JUMP END      
    {tab}
                    
    JUMP WHILE\n\nEND, CLEAR\n"
    ));

    Ok(())
}
