use failure::Error;
use filter::{Expression, Filter};

/*
grammar:

query: if <expr> then <expr>
     | <expr>
expr: string
    | not string
    | <expr> and <expr>
    | <expr> or <expr>
*/

fn new_match(bytes: &[u8]) -> Expression {
  Expression::Match(String::from_utf8(bytes.to_vec()).unwrap())
}

named!(match_expr<&[u8], Expression>,
  map!(
    ws!(delimited!(char!('\''), take_until!("'"), char!('\''))),
    new_match
  )
);

fn new_not(expr: Expression) -> Expression {
  Expression::Not(Box::new(expr))
}

named!(not_expr<&[u8], Expression>,
  map!(
    ws!(preceded!(tag!("not"), expr)),
    new_not
  )
);

fn new_if(t: (Expression, Expression)) -> Expression {
  Expression::If(Box::new(t.0), Box::new(t.1), Box::new(Expression::Empty))
}

named!(if_expr<&[u8], Expression>,
  map!(
    ws!(do_parse!(
      tag!("if") >>
      cond: expr >>
      tag!("then") >>
      subexpr: expr >>
      (cond, subexpr)
    )),
    new_if
  )
);

named!(expr<&[u8], Expression>, alt!(match_expr | not_expr | if_expr));

named!(binary_op<&[u8], Expression>,
  do_parse!(
    init: expr >>
    fold: fold_many0!(
      ws!(pair!(alt!(tag!("or") | tag!("and")), expr)),
      init,
      |acc: Expression, (op, item): (&[u8], Expression)| {
        if op == b"or" {
          Expression::Or(Box::new(acc), Box::new(item))
        } else {
          Expression::And(Box::new(acc), Box::new(item))
        }
      }
    ) >>
    (fold)
  )
);

named!(root_expr<&[u8], Expression>,
  terminated!(
    alt!(binary_op | expr),
    eof!()
  )
);

fn parse_expression(expression: &str) -> Result<Expression, Error> {
  let result = root_expr(expression.as_bytes()).to_result()?;
  Ok(result)
}

pub fn parse_filter(filters: &[String]) -> Result<Filter, Error> {
  let mut expression = Expression::Empty;
  for filter in filters {
    let subexpr = parse_expression(filter)?;
    expression = Expression::And(Box::new(expression), Box::new(subexpr));
  }
  Ok(Filter::new(expression))
}
