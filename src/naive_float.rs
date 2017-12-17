use float_cmp::{ApproxEqUlps, ApproxOrdUlps};
use serde::ser::{Serialize, Serializer};
use std::cmp::Ordering;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct NaiveFloat {
  inner: f64
}

impl PartialEq for NaiveFloat {
  fn eq(&self, other: &NaiveFloat) -> bool {
    self.inner.approx_eq_ulps(&other.inner, 2)
  }
}
impl Eq for NaiveFloat {}

impl Ord for NaiveFloat {
  fn cmp(&self, other: &NaiveFloat) -> Ordering {
    self.inner.approx_cmp(&other.inner, 2)
  }
}
impl PartialOrd for NaiveFloat {
  fn partial_cmp(&self, other: &NaiveFloat) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl FromStr for NaiveFloat {
  type Err = ::std::num::ParseFloatError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    f64::from_str(s).map(|f| NaiveFloat { inner: f })
  }
}

impl Serialize for NaiveFloat {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
  {
    self.inner.serialize(serializer)
  }
}
