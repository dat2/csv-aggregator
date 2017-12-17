use csv::StringRecord;

#[derive(Clone, Debug)]
pub struct Filter {
  expression: Expression
}

impl Filter {
  pub fn new(expression: Expression) -> Filter {
    Filter { expression: expression }
  }

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
        record.iter().any(|field| field.contains(substring))
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
