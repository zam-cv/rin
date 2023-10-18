use crate::{
    expr::{parse_expr, resolve_expr},
    parser::{Global, Rule, DEFAULT_VALUE, STORE},
    statements::functions::returns_value,
};
use pest::iterators::Pair;
use anyhow::{anyhow, Result};

pub fn variable(pair: Pair<Rule>, global: &mut Global, output: &mut String) -> Result<()> {
    let mut inner = pair.into_inner();

    let ident = inner
        .next()
        .ok_or_else(|| anyhow!("Missing identifier"))?
        .as_str()
        .to_string();

    let (type_value, value_pair) = if 2 == inner.clone().count() {
        let type_value = inner
            .next()
            .ok_or_else(|| anyhow!("Missing type"))?
            .as_str()
            .to_string();

        let value_pair = inner.next().ok_or_else(|| anyhow!("Missing value"))?;

        (type_value, value_pair)
    } else {
        (
            "DEC".to_string(),
            inner.next().ok_or_else(|| anyhow!("Missing value"))?,
        )
    };

    match value_pair.as_rule() {
        Rule::expr => {
            let expr = parse_expr(value_pair.into_inner());
            resolve_expr(expr, &ident, &type_value, global, output, STORE)?;
        }
        Rule::function_call => returns_value(value_pair, &ident, output)?,
        _ => return Err(anyhow!("Invalid value assignment")),
    }

    global.variables.insert(ident, (type_value, DEFAULT_VALUE));

    Ok(())
}
