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
    functions: &mut BTreeMap<String, String>,
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
                while_loop(pair, variables, output, count_mul, count_div, functions)?;
            }
            Rule::function => {
                let mut inner = pair.into_inner();

                let ident = inner
                    .next()
                    .ok_or_else(|| anyhow!("Missing identifier"))?
                    .as_str();

                let mut block_output = String::new();

                resolve_instructions(
                    inner,
                    variables,
                    &mut block_output,
                    count_mul,
                    count_div,
                    functions,
                )?;

                let tab = block_output
                    .lines()
                    .map(|line| format!("\t{}", line))
                    .collect::<Vec<String>>()
                    .join("\n");

                let mut func = String::new();
                func.push_str(&format!("{ident},\tHEX     000\n{tab}\n\tJumpI   {ident}"));
                functions.insert(ident.to_string(), func);
            }
            Rule::function_call => {
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
                        output.push_str(&format!("JNS {ident}\n"));
                    }
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
    let mut functions: BTreeMap<String, String> = BTreeMap::new();
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
        &mut functions,
    )?;

    output.push_str("\nHALT\n\n");

    for (_, func) in functions {
        output.push_str(&format!("{}\n\n", func));
    }

    for (ident, (stype, value)) in variables {
        output.push_str(&format!("{},   {} {}\n", ident, stype, value));
    }

    output.pop();
    Ok(output)
}
