use crate::{
    operations::{add, divide, multiply, subtract},
    parser::{Expr, Global, Op, Rule, DEFAULT_VALUE},
    statements::functions::returns_value,
};
use anyhow::{anyhow, Result};
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
            Rule::deref => Expr::Deref(primary.as_str().to_string().replace("*", "")),
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

pub fn resolve_sub_expr(
    token: Expr,
    type_value: &String,
    global: &mut Global,
    output: &mut String,
    type_store: &str,
) -> Result<String> {
    Ok(match token {
        Expr::Value(value) => set_number(&value, &type_value, &mut global.variables),
        Expr::Variable(name) => name,
        Expr::Deref(ptr_name) => {
            global.variables.insert(
                format!("{ptr_name}_value"),
                (type_value.clone(), DEFAULT_VALUE),
            );
            output.push_str(&format!("LOADI {ptr_name}\nSTORE {ptr_name}_value\n"));
            format!("{ptr_name}_value")
        }
        Expr::BinOp { lhs, op, rhs } => {
            let r = String::from("R");

            resolve_expr(
                Expr::BinOp { lhs, op, rhs },
                &r,
                &type_value,
                global,
                output,
                type_store,
            )?;

            r
        }
    })
}

pub fn resolve_expr(
    expression: Expr,
    name: &String,
    type_value: &String,
    global: &mut Global,
    output: &mut String,
    type_store: &str,
) -> Result<()> {
    let name = name.clone();
    let type_value = type_value.clone();

    match expression {
        Expr::Value(value) => {
            let n = set_number(&value, &type_value, &mut global.variables);
            output.push_str(&format!("LOAD {n}\n{type_store} {name}\n\n"));
        }
        Expr::Variable(var) => {
            output.push_str(&format!("LOAD {var}\n{type_store} {name}\n"));
            global.variables.insert(name, (type_value, DEFAULT_VALUE));
        }
        Expr::Deref(ptr_name) => {
            output.push_str(&format!("LOADI {ptr_name}\n{type_store} {name}\n"));
            global.variables.insert(name, (type_value, DEFAULT_VALUE));
        }
        Expr::BinOp { lhs, op, rhs } => {
            let lhs = resolve_sub_expr(*lhs, &type_value, global, output, type_store)?;

            let rhs = resolve_sub_expr(*rhs, &type_value, global, output, type_store)?;

            let result = match op {
                Op::Add => add(&lhs, &rhs),
                Op::Subtract => subtract(&lhs, &rhs),
                Op::Multiply => multiply(&lhs, &rhs, &mut global.counts.mul),
                Op::Divide => divide(&lhs, &rhs, &mut global.counts.div),
            };

            output.push_str(&result);
            output.push_str(&format!("\n{type_store} {name}\n"));
        }
    };

    Ok(())
}

pub fn assign(
    pair: pest::iterators::Pair<Rule>,
    global: &mut Global,
    output: &mut String,
    type_store: &str,
) -> Result<()> {
    let mut inner = pair.into_inner();

    let ident = inner
        .next()
        .ok_or_else(|| anyhow!("Missing identifier"))?
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
            let type_name = global
                .variables
                .get(&ident)
                .ok_or_else(|| anyhow!("Variable not found"))?
                .0
                .clone();

            resolve_expr(expr, &ident, &type_name, global, output, type_store)?;
        }
        Rule::function_call => returns_value(value_pair, &ident, output)?,
        _ => return Err(anyhow!("Invalid value assignment")),
    }

    Ok(())
}
