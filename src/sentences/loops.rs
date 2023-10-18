use crate::parser::{resolve_instructions, Global, Rule};
use anyhow::{anyhow, Result};

pub fn while_loop(
    pair: pest::iterators::Pair<Rule>,
    global: &mut Global,
    output: &mut String,
) -> Result<()> {
    let mut inner = pair.into_inner();

    let var_n = inner
        .next()
        .ok_or_else(|| anyhow!("Missing valueA"))?
        .as_str();

    let condition = inner
        .next()
        .ok_or_else(|| anyhow!("Missing condition"))?
        .as_str()
        .to_string();

    let var_m = inner
        .next()
        .ok_or_else(|| anyhow!("Missing valueB"))?
        .as_str();

    let mut block = String::new();
    resolve_instructions(inner, global, &mut block)?;

    let operator = match condition.as_str() {
        "<" => "800",
        "!=" => "400",
        ">" => "000",
        _ => return Err(anyhow!("Invalid condition")),
    };

    let loops = global.counts.whiles;

    output.push_str(&format!(
        "CLEAR\nSTORE R\n\nBEGIN_{loops}, JUMP NEXT_{loops}
INIT_{loops},	CLEAR\n\n{block}\nNEXT_{loops},	LOAD {var_n}\n\tSUBT {var_m}\n\tSKIPCOND {operator}\n\tJUMP INIT_{loops}\n"));

    global.counts.whiles += 1;
    Ok(())
}

pub fn loopp(
    pair: pest::iterators::Pair<Rule>,
    global: &mut Global,
    output: &mut String,
) -> Result<()> {
    let inner = pair.into_inner();
    let mut block = String::new();

    resolve_instructions(inner, global, &mut block)?;
    let loops = global.counts.loops;

    output.push_str(&format!(
        "LOOP_{loops},	CLEAR\n\n{block}\nJUMP LOOP_{loops}\n"
    ));

    Ok(())
}