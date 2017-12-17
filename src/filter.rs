use csv::StringRecord;
use failure::Error;
use nom::space;

#[derive(Clone, Debug)]
pub struct Filter {
  expression: Expression
}

impl Filter {
  pub fn matches(&self, record: &StringRecord) -> bool {
    self.expression.matches(record)
  }
}

#[derive(Clone, Debug)]
pub enum Expression {
  Empty,
  Match(String),
  Not(Box<Expression>),
  Or(Box<Expression>, Box<Expression>),
  And(Box<Expression>, Box<Expression>),
  If(Box<Expression>, Box<Expression>, Box<Expression>)
}

impl Expression {
  pub fn matches(&self, record: &StringRecord) -> bool {
    match *self {
      Expression::Empty => true,
      Expression::Match(ref substring) => {
        record.iter().find(|&field| field.contains(substring)).is_some()
      },
      Expression::Not(ref subexpr) => {
        !subexpr.matches(record)
      }
      Expression::Or(ref left, ref right) => {
        left.matches(record) || right.matches(record)
      },
      Expression::And(ref left, ref right) => {
        left.matches(record) && right.matches(record)
      },
      Expression::If(ref cond, ref true_branch, ref false_branch) => {
        if cond.matches(record) {
          true_branch.matches(record)
        } else {
          false_branch.matches(record)
        }
      }
    }
  }
}

fn new_match(t: (Option<&[u8]>, &[u8])) -> Expression {
  let subexpr = Expression::Match(String::from_utf8(t.1.to_vec()).unwrap());
  if let Some(_) = t.0 {
    Expression::Not(Box::new(subexpr))
  } else {
    subexpr
  }
}

named!(match_expr<&[u8], Expression>,
  map!(
    do_parse!(
      tag: opt!(tag!("not")) >>
      many0!(space) >>
      expr: delimited!(char!('\''), take_until!("'"), char!('\'')) >>
      (tag, expr)
    ),
    new_match
  )
);

fn new_if(t: (Expression, Expression)) -> Expression {
  Expression::If(Box::new(t.0), Box::new(t.1), Box::new(Expression::Empty))
}

named!(if_expr<&[u8], Expression>,
  map!(
    do_parse!(
      tag!("if") >>
      many1!(space) >>
      cond: expr >>
      tag!("then") >>
      many1!(space) >>
      subexpr: expr >>
      (cond, subexpr)
    ),
    new_if
  )
);

named!(expr<&[u8], Expression>,
  do_parse!(
    expr: alt!(if_expr | match_expr) >>
    many0!(space) >>
    (expr)
  )
);

named!(or_expr<&[u8], Expression>,
  do_parse!(
    left: expr >>
    fold: fold_many0!(
      do_parse!(
        tag!("or") >>
        many1!(space) >>
        expr: expr >>
        (expr)
      ),
      left,
      |acc: Expression, item: Expression| {
        Expression::Or(Box::new(acc), Box::new(item))
      }
    ) >>
    (fold)
  )
);

named!(and_expr<&[u8], Expression>,
  do_parse!(
    left: expr >>
    fold: fold_many0!(
      do_parse!(
        tag!("and") >>
        many1!(space) >>
        expr: expr >>
        (expr)
      ),
      left,
      |acc: Expression, item: Expression| {
        Expression::And(Box::new(acc), Box::new(item))
      }
    ) >>
    (fold)
  )
);

named!(root_expr<&[u8], Expression>,
  do_parse!(
    expr: dbg_dmp!( alt!(and_expr | or_expr | expr) ) >>
    eof!() >>
    (expr)
  )
);

pub fn parse_expression(expression: &str) -> Result<Expression, Error> {
  let result = root_expr(expression.as_bytes()).to_result()?;
  Ok(result)
}

pub fn parse_filter(filters: &[String]) -> Result<Filter, Error> {
  let mut expression = Expression::Not(Box::new(Expression::Empty));
  for filter in filters {
    let subexpr = parse_expression(filter)?;
    expression = Expression::Or(Box::new(expression), Box::new(subexpr));
  }
  Ok(Filter { expression: expression })
}
