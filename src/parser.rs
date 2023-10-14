use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

pub fn generate(code: &str) -> String {
    let parsed = MyParser::parse(Rule::program, &code).unwrap_or_else(|e| {
        panic!("Parsing error: {}", e);
    });

    for pair in parsed {
        match pair.as_rule() {
            Rule::statement => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::ident => println!("Identificador encontrado: {}", inner_pair.as_str()),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    String::from("Hello, world!")
}