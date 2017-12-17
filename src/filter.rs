use csv::StringRecord;

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

fn new_match(bytes: &[u8]) -> Expression {
  Expression::Match(String::from_utf8(bytes.to_vec()).unwrap())
}

named!(match_expr <&[u8], Expression>,
  map!(
    delimited!(tag!("'"), take_until!("'"), tag!("'")),
    new_match
  )
);

pub fn parse_expression(expression: &str) -> Expression {
  Expression::Empty
}

pub fn parse_filter(filters: &[String]) -> Filter {
  let mut expression = Expression::Empty;
  for filter in filters {
    let subexpr = parse_expression(filter);
    expression = Expression::Or(Box::new(expression), Box::new(subexpr));
  }
  Filter { expression: expression }
}
