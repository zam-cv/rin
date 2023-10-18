use crate::{
    expr::assign,
    sentences::{
        iff::iff,
        loops::{loopp, while_loop},
    },
    statements::{
        functions::{function, function_call},
        variable::variable,
    },
};
use anyhow::Result;
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
    pub whiles: u64,
    pub ifs: u64,
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
                variable(pair, global, output)?;
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
            Rule::loopp => {
                loopp(pair, global, output)?;
            }
            Rule::iff => {
                iff(pair, global, output)?;
            }
            Rule::function_call => {
                function_call(pair, output)?;
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
        whiles: 1,
        loops: 1,
        ifs: 1,
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
\nSUB,	HEX 000\n\tLOAD R\n\tSUBT ONE\n\tSTORE R\n\tLOAD A\n\tSUBT B\n\tJUMPI SUB"
            ),
        );
    }

    if global.counts.mul > 1 {
        global.functions.insert(
            "MUL".to_string(),
            format!(
                "MUL_ADD,\tHEX 000\n\tLOAD R\n\tADD B\n\tSTORE R\n\n\tLOAD COUNT\n\tSUBT ONE\n\tSTORE COUNT\n\tJUMPI MUL_ADD"
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
