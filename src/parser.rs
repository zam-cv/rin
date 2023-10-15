use crate::expr::{assign, parse_expr, resolve_expr};
use crate::functions::returns_value;
use crate::loops::while_loop;
use anyhow::{anyhow, Result};
use pest::Parser;
use std::collections::BTreeMap;

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

#[derive(Debug)]
pub enum Expr {
    Value(u16),
    Variable(String),
    Deref(String),
    BinOp {
        lhs: Box<Expr>,
        op: Op,
        rhs: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
}

pub const STORE: &str = "STORE";
pub const STOREI: &str = "STOREI";
pub const DEFAULT_VALUE: u16 = 0;
pub const DEFAULT_VARIABLES: [(&str, &str, u16); 6] = [
    ("COUNT", "DEC", DEFAULT_VALUE),
    ("ONE", "DEC", 1),
    ("R", "DEC", DEFAULT_VALUE),
    ("K", "DEC", DEFAULT_VALUE),
    ("J", "DEC", DEFAULT_VALUE),
    ("POW", "DEC", DEFAULT_VALUE),
];

pub fn resolve_instructions(
    parsed: pest::iterators::Pairs<Rule>,
    variables: &mut BTreeMap<String, (String, u16)>,
    output: &mut String,
    count_mul: &mut u64,
    count_div: &mut u64,
) -> Result<()> {
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
                    Rule::deref => {
                        let mut inner = value_pair.into_inner();

                        let ptr_name = inner
                            .next()
                            .ok_or_else(|| anyhow!("Missing identifier"))?
                            .as_str()
                            .to_string();

                        output.push_str(&format!("CLEAR\nLOADI {ptr_name}\nSTORE {ident}\n\n"));
                    }
                    Rule::expr => {
                        let expr = parse_expr(value_pair.into_inner());
                        output.push_str("CLEAR\nSTORE R\n");

                        resolve_expr(
                            expr,
                            &ident,
                            &type_value,
                            variables,
                            output,
                            count_mul,
                            count_div,
                            STORE,
                        )?;
                    }
                    Rule::function_call => returns_value(value_pair, &ident, output)?,
                    _ => return Err(anyhow!("Invalid value assignment")),
                }

                variables.insert(ident, (type_value, DEFAULT_VALUE));
            }
            Rule::reassignment => {
                assign(pair, variables, output, count_mul, count_div, STORE)?;
            }
            Rule::dereference => {
                assign(pair, variables, output, count_mul, count_div, STOREI)?;
            }
            Rule::while_loop => {
                while_loop(pair, variables, output, count_mul, count_div)?;
            }
            Rule::function_call => {
                let mut inner = pair.into_inner();

                let ident = inner
                    .next()
                    .ok_or_else(|| anyhow!("Missing identifier"))?
                    .as_str();

                let params = inner
                    .next()
                    .ok_or_else(|| anyhow!("Missing parameters"))?
                    .as_str();

                match ident {
                    "print" => {
                        output.push_str(&format!("LOAD {params}\nOUTPUT\n"));
                    }
                    _ => return Err(anyhow!("Unsupported function: {ident}")),
                }
            }
            _ => {}
        }
    }

    Ok(())
}

pub fn generate(code: &str) -> Result<String> {
    let mut count_mul = 1;
    let mut count_div = 1;
    let mut variables: BTreeMap<String, (String, u16)> = BTreeMap::new();
    let mut output = String::new();

    for (token, stype, value) in DEFAULT_VARIABLES.iter() {
        variables.insert(token.to_string(), (stype.to_string(), *value));
    }

    let parsed = MyParser::parse(Rule::program, &code).unwrap_or_else(|e| {
        panic!("Parsing error: {}", e);
    });

    resolve_instructions(
        parsed,
        &mut variables,
        &mut output,
        &mut count_mul,
        &mut count_div,
    )?;

    output.push_str("\nHALT\n\n");

    for (ident, (stype, value)) in variables {
        output.push_str(&format!("{},   {} {}\n", ident, stype, value));
    }

    output.pop();
    Ok(output)
}
