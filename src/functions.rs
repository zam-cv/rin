use crate::parser::{resolve_instructions, Rule, Global};
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

pub fn create_function(
    ident: String,
    block: pest::iterators::Pairs<Rule>,
    global: &mut Global,
) -> Result<()> {
    let mut block_output = String::new();

    resolve_instructions(
        block,
        global,
        &mut block_output,
    )?;

    let tab = block_output
        .lines()
        .map(|line| format!("\t{}", line))
        .collect::<Vec<String>>()
        .join("\n");

    let mut func = String::new();
    func.push_str(&format!("{ident},\tHEX     000\n{tab}\n\tJumpI   {ident}"));
    global.functions.insert(ident.to_string(), func);

    Ok(())
}

pub fn function(
    pair: pest::iterators::Pair<Rule>,
    global: &mut Global,
) -> Result<()> {
    let mut block = pair.into_inner();

    let ident = block
        .next()
        .ok_or_else(|| anyhow!("Missing identifier"))?
        .as_str();

    create_function(
        ident.to_string(),
        block,
        global
    )?;

    Ok(())
}
