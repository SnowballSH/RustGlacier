use glacier_lang::glacier_parser::parse;

fn main() {
    dbg!(
        "{:>}",
        parse(
            "\
    if 1 + 1 == 2
        println(\"yep\")
    else
        println(\"nah\")
    end"
        ).expect("no")
    );
}
