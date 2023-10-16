use crate::expr::{assign, parse_expr, resolve_expr};
use crate::functions::{function, returns_value};
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

pub struct Counts {
    pub mul: u64,
    pub div: u64,
    pub loops: u64,
}

pub struct Global {
    pub variables: BTreeMap<String, (String, u16)>,
    pub functions: BTreeMap<String, String>,
    pub counts: Counts,
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
    global: &mut Global,
    output: &mut String,
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
                        output.push_str("CLEAR\nSTORE R\n");

                        resolve_expr(expr, &ident, &type_value, global, output, STORE)?;
                    }
                    Rule::function_call => returns_value(value_pair, &ident, output)?,
                    _ => return Err(anyhow!("Invalid value assignment")),
                }

                global.variables.insert(ident, (type_value, DEFAULT_VALUE));
            }
            Rule::reassignment => {
                assign(pair, global, output, STORE)?;
            }
            Rule::dereference => {
                assign(pair, global, output, STOREI)?;
            }
            Rule::while_loop => {
                while_loop(pair, global, output)?;
            }
            Rule::function => {
                function(pair, global)?;
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
    let mut output = String::new();

    let counts = Counts {
        mul: 1,
        div: 1,
        loops: 1,
    };

    let mut global = Global {
        variables: BTreeMap::new(),
        functions: BTreeMap::new(),
        counts,
    };

    for (token, stype, value) in DEFAULT_VARIABLES.iter() {
        global
            .variables
            .insert(token.to_string(), (stype.to_string(), *value));
    }

    let parsed = MyParser::parse(Rule::program, &code).unwrap_or_else(|e| {
        panic!("Parsing error: {}", e);
    });

    resolve_instructions(parsed, &mut global, &mut output)?;

    output.push_str("\nHALT\n\n");

    for (_, func) in global.functions {
        output.push_str(&format!("{}\n\n", func));
    }

    for (ident, (stype, value)) in global.variables {
        output.push_str(&format!("{},   {} {}\n", ident, stype, value));
    }

    output.pop();
    Ok(output)
}
