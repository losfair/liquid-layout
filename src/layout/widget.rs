use anyhow::Result;

use super::{measure::Measure, prop::Prop};

pub trait RawWidget<'a> {
  fn measures(&self) -> Vec<Measure<'a>>;
  fn constraints(&self) -> Vec<Prop<'a>>;
  fn paint(self: Box<Self>, measures: &[f64]) -> Result<()>;
}
