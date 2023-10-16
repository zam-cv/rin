use crate::parser::{resolve_instructions, Global, Rule};
use anyhow::{anyhow, Result};

pub fn while_loop(
    pair: pest::iterators::Pair<Rule>,
    global: &mut Global,
    output: &mut String,
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

    resolve_instructions(inner, global, &mut block_output)?;

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

    let loops = global.counts.loops;

    output.push_str(&format!(
        "WHILE_{loops},\tLOAD {var_a}\n\tSUBT {var_b}\n\tSKIPCOND {operator}\n\tJUMP END
\tJNS BLOCK_{loops}\n\tJUMP WHILE_{loops}\n\nEND, CLEAR\n"));

    global.functions.insert(
        format!("WHILE_{loops}"),
        format!("BLOCK_{loops},\tHEX\t000\n{tab}\n\tJumpI\tBLOCK_{loops}"),
    );

    global.counts.loops += 1;
    Ok(())
}
