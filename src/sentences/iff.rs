use crate::parser::{resolve_instructions, Global, Rule};
use anyhow::{anyhow, Result};

pub fn iff(
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

    let mut block = String::new();
    resolve_instructions(inner, global, &mut block)?;

    let operator = match condition.as_str() {
        "<" => "000",
        "==" => "400",
        ">" => "800",
        _ => return Err(anyhow!("Invalid condition")),
    };

    let ifs = global.counts.ifs;
    output.push_str(&format!("LOAD {var_a}\nSUBT {var_b}\nSKIPCOND {operator}\nJUMP BREAK_{ifs}\n\n{block}BREAK_{ifs},	CLEAR\n\n"));

    global.counts.ifs += 1;

    Ok(())
}
