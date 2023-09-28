use inator::{ignore, opt, Parser};

fn comma_separated(p: Parser<char>) -> Parser<char> {
    Parser::empty() | (p + (on(',', "list_sep") + p).star() + ignore(',').optional())
}

fn string() -> Parser<char> {
    on('"', "begin_string") >> string_contents() >> ignore('"')
}

fn string_contents() -> Parser<char> {
    todo!()
}

fn key_value() -> Parser<char> {
    string() + on(':', "key_value_sep") + value()
}

fn object() -> Parser<char> {
    on('{', "begin_object") + comma_separated(object_item()) + ignore('}')
}

fn object_item() -> Parser<char> {
    todo!()
}

fn array() -> Parser<char> {
    on('[', "begin_array") + comma_separated(array_item()) + ignore(']')
}

fn array_item() -> Parser<char> {
    todo!()
}

fn parser() -> Parser<char> {
    todo!()
}

fn main() -> std::io::Result<()> {
    std::fs::write("src/parser.rs", parser().compile().into_source("json"))
}
