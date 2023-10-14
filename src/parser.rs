use anyhow::{anyhow, Result};
use pest::iterators::Pairs;
use pest::pratt_parser::PrattParser;
use pest::Parser;
use std::collections::BTreeMap;

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

#[derive(Debug)]
pub enum Expr {
    Value(u16),
    Variable(String),
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

lazy_static::lazy_static! {
  static ref PRATT_PARSER: PrattParser<Rule> = {
      use pest::pratt_parser::{Assoc::*, Op};
      use Rule::*;

      PrattParser::new()
          .op(Op::infix(add, Left) | Op::infix(subtract, Left))
          .op(Op::infix(multiply, Left) | Op::infix(divide, Left))
  };
}

pub fn parse_expr(pairs: Pairs<Rule>) -> Expr {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::integer => Expr::Value(primary.as_str().parse::<u16>().unwrap()),
            Rule::ident => Expr::Variable(primary.as_str().to_string()),
            rule => unreachable!("Expr::parse expected primary, found {:?}", rule),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::add => Op::Add,
                Rule::subtract => Op::Subtract,
                Rule::multiply => Op::Multiply,
                Rule::divide => Op::Divide,
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };
            Expr::BinOp {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            }
        })
        .parse(pairs)
}

pub fn resolve_expr(
    expr: Expr,
    ident: &String,
    type_value: &String,
    mut variables: &mut BTreeMap<String, (String, u16)>,
    output: &mut String,
    count_mul: &mut u64,
    count_div: &mut u64,
) -> Result<()> {
    let ident = ident.clone();
    let type_value = type_value.clone();

    match expr {
        Expr::Value(value) => {
            variables.insert(ident, (type_value, value as u16));
        }
        Expr::Variable(var) => {
            output.push_str(&format!("\nLOAD {}\nSTORE {}\nCLEAR\n", var, ident));
            variables.insert(ident, (type_value, DEFAULT_VALUE));
        }
        Expr::BinOp { lhs, op, rhs } => {
            let lhs = match *lhs {
                Expr::Value(value) => set_number(&value, &type_value, &mut variables),
                Expr::Variable(ident) => ident,
                Expr::BinOp { lhs, op, rhs } => {
                    let r = String::from("R");

                    resolve_expr(
                        Expr::BinOp { lhs, op, rhs },
                        &r,
                        &type_value,
                        &mut variables,
                        output,
                        count_mul,
                        count_div,
                    )?;

                    r
                }
            };

            let rhs = match *rhs {
                Expr::Value(value) => set_number(&value, &type_value, &mut variables),
                Expr::Variable(ident) => ident,
                Expr::BinOp { lhs, op, rhs } => {
                    let r = String::from("R");

                    resolve_expr(
                        Expr::BinOp { lhs, op, rhs },
                        &r,
                        &type_value,
                        &mut variables,
                        output,
                        count_mul,
                        count_div,
                    )?;

                    r
                }
            };

            let result = match op {
                Op::Add => add(&lhs, &rhs),
                Op::Subtract => subtract(&lhs, &rhs),
                Op::Multiply => multiply(&lhs, &rhs, count_mul),
                Op::Divide => divide(&lhs, &rhs, count_div),
            };

            output.push_str(&result);
            output.push_str(&format!("STORE {}\nCLEAR\n", &ident));
        }
    };

    Ok(())
}

fn set_number(
    number: &u16,
    type_value: &String,
    variables: &mut BTreeMap<String, (String, u16)>,
) -> String {
    let ident = format!("N{}", number);
    variables.insert(format!("N{}", number), (type_value.clone(), number.clone()));
    return ident;
}

const DEFAULT_VALUE: u16 = 0;
static DEFAULT_VARIABLES: [(&str, &str, u16); 6] = [
    ("COUNT", "DEC", DEFAULT_VALUE),
    ("ONE", "DEC", 1),
    ("R", "DEC", DEFAULT_VALUE),
    ("K", "DEC", DEFAULT_VALUE),
    ("J", "DEC", DEFAULT_VALUE),
    ("POW", "DEC", DEFAULT_VALUE),
];

fn multiply(a: &String, b: &String, count: &mut u64) -> String {
    let value = format!(
        "\nLOAD {a}\nSTORE COUNT\n\nLOOP_{count},	LOAD R
    ADD {b}
    STORE R
           
    LOAD COUNT
    SUBT ONE
    STORE COUNT
            
    SKIPCOND 400
    JUMP LOOP_{count}\n\nLOAD R\n"
    );
    *count += 1;
    value
}

fn add(a: &String, b: &String) -> String {
    format!("\nLOAD {a}\nADD {b}\n")
}

fn subtract(a: &String, b: &String) -> String {
    format!("\nLOAD {a}\nSUBT {b}\n")
}

fn divide(a: &String, b: &String, count: &mut u64) -> String {
    let value = format!(
        "CLEAR\nSTORE J\nSTORE POW\nLOAD {a}\nSTORE K\nCLEAR
    
OUTER_{count}, LOAD K
    SKIPCOND 800
    JUMP DONE_{count}
    LOAD ONE
    STORE POW
    LOAD {b}
    STORE J
    
INNER_{count}, LOAD J
    ADD J
    SUBT K
    SKIPCOND 000
    JUMP AFTIN_{count}
    LOAD J
    ADD J
    STORE J
    LOAD POW
    ADD POW
    STORE POW
    JUMP INNER_{count}
    
AFTIN_{count}, LOAD K
    SUBT J
    STORE K
    LOAD R
    ADD POW
    STORE R
    JUMP OUTER_{count}
    
DONE_{count}, LOAD K
    SKIPCOND 000
    JUMP DISP_{count}
    LOAD R
    SUBT ONE

DISP_{count}, LOAD R\n\n"
    );
    *count += 1;
    value
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
                    Rule::integer => {
                        let value = value_pair.as_str().parse::<u16>()?;
                        variables.insert(ident, (type_value, value));
                    }
                    Rule::expr => {
                        let expr = parse_expr(value_pair.into_inner());
                        output.push_str("CLEAR\nSTORE R\n\n");

                        resolve_expr(
                            expr,
                            &ident,
                            &type_value,
                            &mut variables,
                            &mut output,
                            &mut count_mul,
                            &mut count_div,
                        )?;

                        variables.insert(ident, (type_value, DEFAULT_VALUE));
                    }
                    Rule::function_call => match value_pair.as_str() {
                        "input()" => {
                            output.push_str(&format!("INPUT\nSTORE {}\nCLEAR\n", &ident));
                            variables.insert(ident, (type_value, DEFAULT_VALUE));
                        }
                        _ => return Err(anyhow!("Unsupported function: {}", value_pair.as_str())),
                    },
                    _ => return Err(anyhow!("Invalid value assignment")),
                }
            }
            Rule::function_call => {
                let mut inner = pair.into_inner();

                let ident = inner
                    .next()
                    .ok_or_else(|| anyhow!("Missing identifier"))?
                    .as_str()
                    .to_string();

                let params = inner
                    .next()
                    .ok_or_else(|| anyhow!("Missing parameters"))?
                    .as_str()
                    .to_string();

                match ident.as_str() {
                    "print" => {
                        output.push_str(&format!("\nLOAD {}\nOUTPUT\n", params));
                    }
                    _ => return Err(anyhow!("Unsupported function: {}", ident)),
                }
            }
            _ => {}
        }
    }

    output.push_str("\nHALT\n\n");

    for (ident, (stype, value)) in variables {
        output.push_str(&format!("{},   {} {}\n", ident, stype, value));
    }

    output.pop();
    Ok(output)
}
