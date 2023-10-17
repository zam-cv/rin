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
    pub ifs: u64
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
    ("A", "DEC", DEFAULT_VALUE),
    ("B", "DEC", DEFAULT_VALUE),
    ("C", "DEC", DEFAULT_VALUE),
    ("R", "DEC", DEFAULT_VALUE),
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
            Rule::if_cond => {
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
                    ">" => "000",
                    "!=" => "400",
                    "<" => "800",
                    _ => return Err(anyhow!("Invalid condition")),
                };
                
                let ifs = global.counts.ifs;
                output.push_str(&format!("LOAD {var_a}\nSTORE A\nLOAD {var_b}\nSTORE B\n\nLOAD A\nSUBT B\nSKIPCOND {operator}\nJNS THEN_1"));
                global.functions.insert(format!("THEN_{ifs}"), format!("THEN_{ifs},\tHEX\t000\n{tab}\n\tJumpI\tTHEN_{ifs}"));
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
        ifs: 1
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

    if global.counts.div > 1 {
        global.functions.insert(
            "DIV".to_string(),
            format!(
                "DIV,	HEX	000\n\tLOAD C\n\tADD B\n\tSTORE C\n\tLOAD R\n\tADD ONE\n\tSTORE R\n\tLOAD A\n\tSUBT C\n\tJUMPI DIV

SUB,	HEX 000\n\tLOAD R\n\tSUBT ONE\n\tSTORE R\n\tLOAD A\n\tSUBT B\n\tJUMPI SUB"
            ),
        );
    }

    for (_, func) in global.functions {
        output.push_str(&format!("{}\n\n", func));
    }

    for (ident, (stype, value)) in global.variables {
        output.push_str(&format!("{},   {} {}\n", ident, stype, value));
    }

    output.pop();
    Ok(output)
}
