use crate::parser::{resolve_instructions, Global, Rule};
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

    resolve_instructions(block, global, &mut block_output)?;
    global.functions.insert(
        ident.to_string(),
        format!("FN_{ident},\tHEX 000\n\n{block_output}\nJUMPI FN_{ident}\n"),
    );

    Ok(())
}

pub fn function(pair: pest::iterators::Pair<Rule>, global: &mut Global) -> Result<()> {
    let mut block = pair.into_inner();

    let ident = block
        .next()
        .ok_or_else(|| anyhow!("Missing identifier"))?
        .as_str();

    create_function(ident.to_string(), block, global)?;

    Ok(())
}

pub fn function_call(pair: pest::iterators::Pair<Rule>, output: &mut String) -> Result<()> {
    let mut inner = pair.into_inner();

    let ident = inner
        .next()
        .ok_or_else(|| anyhow!("Missing identifier"))?
        .as_str();

    match ident {
        "print" => {
            let params = inner
                .next()
                .ok_or_else(|| anyhow!("Missing parameters"))?
                .as_str();

            output.push_str(&format!("LOAD {params}\nOUTPUT\n"));
        }
        _ => {
            output.push_str(&format!("JNS FN_add\n"));
        }
    };

    Ok(())
}