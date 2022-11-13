use lazy_static::*;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest::prec_climber::*;
use pest_derive::*;

use ast::*;
use Rule::*;

pub mod ast;

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use Assoc::*;

        PrecClimber::new(vec![
            Operator::new(log_or, Left),
            Operator::new(log_and, Left),
            Operator::new(dbeq, Left) | Operator::new(neq, Left),
            Operator::new(gt, Left)
                | Operator::new(lt, Left)
                | Operator::new(gteq, Left)
                | Operator::new(lteq, Left),
            Operator::new(add, Left) | Operator::new(sub, Left),
            Operator::new(mul, Left) | Operator::new(div, Left) | Operator::new(modulo, Left),
        ])
    };
}

#[derive(Parser)]
#[grammar = "parser/glacier.pest"]
pub struct GlacierParser;

fn infix<'a>(lhs: Expression<'a>, op: Pair<'a, Rule>, rhs: Expression<'a>) -> Expression<'a> {
    Expression::Infix(Box::new(Infix {
        left: lhs,
        operator: op.as_str(),
        right: rhs,
        pos: op.as_span().into(),
    }))
}

fn others(pair: Pair<Rule>) -> Expression {
    match pair.as_rule() {
        Rule::string_literal => Expression::String_(String_ {
            value: {
                let mut s = String::new();
                let mut escape_mode = false;
                for c in pair.as_str()[1..pair.as_str().len() - 1].chars() {
                    if escape_mode {
                        match c {
                            'n' => s.push('\n'),
                            't' => s.push('\t'),
                            'r' => s.push('\r'),
                            '0' => s.push('\0'),
                            '"' => s.push('"'),
                            '\'' => s.push('\''),
                            '\\' => s.push('\\'),
                            _ => (),
                        }
                        escape_mode = false;
                    } else if c == '\\' {
                        escape_mode = true;
                    } else {
                        s.push(c);
                    }
                }
                s
            },
            pos: pair.as_span().into(),
        }),

        Rule::integer => Expression::Int(Integer {
            value: pair.as_str(),
            pos: pair.as_span().into(),
        }),

        Rule::float => Expression::Float(Float {
            value: pair.as_str(),
            pos: pair.as_span().into(),
        }),

        Rule::array => {
            let inner = pair.clone().into_inner();
            Expression::Array(Array {
                values: inner.map(parse_expression).collect(),
                pos: pair.as_span().into(),
            })
        }

        Rule::false_expr => Expression::Bool(Bool {
            value: false,
            pos: pair.as_span().into(),
        }),
        Rule::true_expr => Expression::Bool(Bool {
            value: true,
            pos: pair.as_span().into(),
        }),

        Rule::identifier => Expression::GetVar(GetVar {
            name: pair.as_str(),
            pos: pair.as_span().into(),
        }),

        Rule::assign => {
            let mut inner = pair.clone().into_inner();
            let name = inner.next().unwrap().as_str();
            let res = inner.next().unwrap();
            Expression::SetVar(Box::new(SetVar {
                name,
                value: parse_expression(res),
                pos: pair.as_span().into(),
            }))
        }

        Rule::condition_if => {
            let mut inner = pair.clone().into_inner();
            let cond = inner.next().unwrap();
            let res = inner.next().unwrap();
            Expression::If(Box::new(If {
                cond: parse_expression(cond),
                body: parse_program(res.into_inner()),
                other: vec![],
                pos: pair.as_span().into(),
            }))
        }

        Rule::condition_ifelse => {
            let mut inner = pair.clone().into_inner();
            let cond = inner.next().unwrap();
            let res = inner.next().unwrap();
            let other = inner.next().unwrap();
            Expression::If(Box::new(If {
                cond: parse_expression(cond),
                body: parse_program(res.into_inner()),
                other: parse_program(other.into_inner()),
                pos: pair.as_span().into(),
            }))
        }

        Rule::while_loop => {
            let mut inner = pair.clone().into_inner();
            let cond = inner.next().unwrap();
            let res = inner.next().unwrap();
            Expression::While(Box::new(While {
                cond: parse_expression(cond),
                body: parse_program(res.into_inner()),
                pos: pair.as_span().into(),
            }))
        }

        Rule::do_block => {
            let mut inner = pair.clone().into_inner();
            let res = inner.next().unwrap();
            Expression::Do(Box::new(Do {
                body: parse_program(res.into_inner()),
                pos: pair.as_span().into(),
            }))
        }

        Rule::prefix => {
            let mut inner: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
            let last = inner.pop().unwrap();
            let mut right = parse_expression(last);

            while let Some(x) = inner.pop() {
                right = Expression::Prefix(Box::new(Prefix {
                    operator: x.as_str(),
                    right,
                    pos: pair.as_span().into(),
                }))
            }

            right
        }
        Rule::suffix => {
            let mut inner = pair.clone().into_inner();
            let res = inner.next().unwrap();
            let mut args_iter = inner;

            let n = args_iter.next().unwrap();
            let mut callee = match n.as_rule() {
                Rule::indexing => Expression::Index(Box::new(Index {
                    callee: parse_expression(res),
                    index: parse_expression(n.into_inner().next().unwrap()),
                    pos: pair.as_span().into(),
                })),
                _ => unreachable!(),
            };

            for xx in args_iter {
                callee = match xx.as_rule() {
                    Rule::indexing => Expression::Index(Box::new(Index {
                        callee,
                        index: parse_expression(xx.into_inner().next().unwrap()),
                        pos: pair.as_span().into(),
                    })),
                    _ => unreachable!(),
                }
            }

            callee
        }

        Rule::expression => climb(pair),
        _ => {
            dbg!(pair.as_rule());
            unreachable!()
        }
    }
}

pub fn climb(pair: Pair<Rule>) -> Expression {
    //dbg!(&pair);
    PREC_CLIMBER.climb(pair.into_inner(), others, infix)
}

fn parse_expression(pair: Pair<Rule>) -> Expression {
    let inner = pair.clone().into_inner().count() != 0;
    let res = if inner && pair.clone().as_rule() == Rule::expression {
        climb(pair)
    } else {
        others(pair)
    };

    res
}

fn parse_statement(pair: Pair<Rule>) -> Statement {
    match pair.as_rule() {
        Rule::expression_stmt => {
            let p = pair.into_inner().next().unwrap();
            let s = p.clone().as_span().into();
            Statement::ExprStmt(ExprStmt {
                expr: parse_expression(p),
                pos: s,
            })
        }
        Rule::debug_print => {
            let p = pair.into_inner().next().unwrap();
            let s = p.clone().as_span().into();
            Statement::DebugPrint(DebugPrint {
                expr: parse_expression(p),
                pos: s,
            })
        }
        Rule::echo_print => {
            let p = pair.into_inner().next().unwrap();
            let s = p.clone().as_span().into();
            Statement::EchoPrint(EchoPrint {
                expr: parse_expression(p),
                pos: s,
            })
        }
        Rule::break_stmt => Statement::Break(Break {
            pos: pair.as_span().into(),
        }),
        Rule::next_stmt => Statement::Next(Next {
            pos: pair.as_span().into(),
        }),

        Rule::suffix_assign => {
            let mut inner = pair.clone().into_inner();
            let res = inner.next().unwrap();
            let mut args_iter = inner.peekable();

            let n = args_iter.next().unwrap();
            let mut callee = match n.as_rule() {
                Rule::indexing => Expression::Index(Box::new(Index {
                    callee: parse_expression(res),
                    index: parse_expression(n.into_inner().next().unwrap()),
                    pos: pair.as_span().into(),
                })),
                _ => unreachable!(),
            };

            while let Some(xx) = args_iter.next() {
                if args_iter.peek().is_some() {
                    callee = match xx.as_rule() {
                        Rule::indexing => Expression::Index(Box::new(Index {
                            callee,
                            index: parse_expression(xx.into_inner().next().unwrap()),
                            pos: pair.as_span().into(),
                        })),
                        _ => unreachable!(),
                    }
                } else {
                    return Statement::PointerAssign(Box::new(PointerAssign {
                        ptr: callee,
                        value: parse_expression(xx),
                        pos: pair.as_span().into(),
                    }));
                }
            }

            unreachable!()
        }

        _ => unreachable!(),
    }
}

fn parse_program(res: Pairs<Rule>) -> Program {
    let mut ast = vec![];
    for pair in res {
        match pair.as_rule() {
            Rule::stmt
            | Rule::expression_stmt
            | Rule::debug_print
            | Rule::echo_print
            | Rule::break_stmt
            | Rule::next_stmt
            | Rule::suffix_assign => ast.push(parse_statement(pair)),
            _ => {}
        }
    }
    ast
}

pub fn parse(code: &str) -> Result<Program, pest::error::Error<Rule>> {
    let res = GlacierParser::parse(Rule::program, code);
    match res {
        Ok(res) => {
            let ast = parse_program(res);
            Ok(ast)
        }
        Err(e) => Err(e),
    }
}
