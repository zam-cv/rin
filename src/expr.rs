use crate::operations::{add, divide, multiply, subtract};
use crate::parser::{Expr, Op, Rule, DEFAULT_VALUE};
use anyhow::Result;
use pest::{iterators::Pairs, pratt_parser::PrattParser};
use std::collections::BTreeMap;

lazy_static::lazy_static! {
  static ref PRATT_PARSER: PrattParser<Rule> = {
      use pest::pratt_parser::{Assoc::*, Op};
      use Rule::*;

      PrattParser::new()
          .op(Op::infix(add, Left) | Op::infix(subtract, Left))
          .op(Op::infix(multiply, Left) | Op::infix(divide, Left))
  };
}

pub fn set_number(
    number: &u16,
    type_value: &String,
    variables: &mut BTreeMap<String, (String, u16)>,
) -> String {
    let name = format!("N{}", number);
    variables.insert(format!("N{}", number), (type_value.clone(), number.clone()));
    return name;
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
    expression: Expr,
    name: &String,
    type_value: &String,
    mut variables: &mut BTreeMap<String, (String, u16)>,
    output: &mut String,
    count_mul: &mut u64,
    count_div: &mut u64,
) -> Result<()> {
    let name = name.clone();
    let type_value = type_value.clone();

    match expression {
        Expr::Value(value) => {
            let n = set_number(&value, &type_value, &mut variables);
            output.push_str(&format!("\nLOAD {n}\nSTORE {name}\nCLEAR\n\n"));
        }
        Expr::Variable(var) => {
            output.push_str(&format!("\nLOAD {}\nSTORE {}\nCLEAR\n", var, name));
            variables.insert(name, (type_value, DEFAULT_VALUE));
        }
        Expr::BinOp { lhs, op, rhs } => {
            let lhs = match *lhs {
                Expr::Value(value) => set_number(&value, &type_value, &mut variables),
                Expr::Variable(name) => name,
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
                Expr::Variable(name) => name,
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
            output.push_str(&format!("STORE {}\nCLEAR\n", &name));
        }
    };

    Ok(())
}
