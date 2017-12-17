use combine::char::{char, spaces, string};
use combine::{between, eof, many1, none_of, try, Parser};
use combine::primitives::Stream;
use combine::state::State;
use failure::Error;
use filter::{Expression, Filter};

/*
term: string | 'not' string
*/
parser! {
  fn term[I]()(I) -> Expression
    where [I: Stream<Item=char>]
  {
    let match_term = between(char('\''), char('\''), many1(none_of("'".chars())))
      .map(Expression::Match);

    let not_term = (string("not").skip(spaces()), match_term.clone())
      .map(|t| Expression::Not(Box::new(t.1)));

    match_term
      .or(not_term)
      .skip(spaces())
  }
}

/*
factor: 'not' '(' factor ')'
      | term 'and' factor
      | term 'or' factor
      | term
*/
parser! {
  fn factor[I]()(I) -> Expression
    where [I: Stream<Item=char>]
  {
    let unary_op = (
      string("not").skip(spaces()),
      between(char('('), char(')'), factor())
    ).map(|t| Expression::Not(Box::new(t.1)));

    let binary_op = (
      term(),
      string("and").or(string("or")).skip(spaces()),
      factor()
    ).map(|t| {
      if t.1 == "and" {
        Expression::And(Box::new(t.0), Box::new(t.2))
      } else {
        Expression::Or(Box::new(t.0), Box::new(t.2))
      }
    });

    try(unary_op)
      .or(try(binary_op))
      .or(term())
      .skip(spaces())
  }
}

/*
query: 'if' factor 'then' factor eof
     | factor eof
*/
parser! {
  fn query[I]()(I) -> Expression
    where [I: Stream<Item=char>]
  {
    let if_query = (
      string("if").skip(spaces()),
      factor(),
      string("then").skip(spaces()),
      factor()
    ).map(|t| Expression::If(Box::new(t.1), Box::new(t.3), Box::new(Expression::Empty)));

    (
      try(if_query).or(factor()),
      eof()
    ).map(|t| t.0)
  }
}

fn parse_expression(expression: &str) -> Result<Expression, Error> {
  query().easy_parse(State::new(expression))
    .map(|t| t.0)
    .map_err(|e| format_err!("{}", e))
}

pub fn parse_filter(filters: &[String]) -> Result<Filter, Error> {
  let mut expression = Expression::Empty;
  for filter in filters {
    let subexpr = parse_expression(filter)?;
    expression = Expression::And(Box::new(expression), Box::new(subexpr));
  }
  Ok(Filter::new(expression))
}
