use anyhow::Result;

use super::{
  context::{LayoutContext, Z3BuildContext},
  prop::Prop,
  widget::RawWidget,
};
use thiserror::Error;

pub struct LayoutBuilder<'a> {
  layout_ctx: &'a LayoutContext,
  widgets: Vec<Box<dyn RawWidget<'a> + 'a>>,
  constraints: Vec<Prop<'a>>,
}

#[derive(Debug)]
pub struct BuildReport<'a> {
  pub satisfied_constraints: Vec<Prop<'a>>,
  pub unsatisfied_constraints: Vec<Prop<'a>>,
}

#[derive(Error, Debug)]
pub enum LayoutUnsatError {
  #[error("provided constraints cannot be satisfied")]
  Unsat,
  #[error("failed to derive a layout under provided constraints")]
  Unknown,
}

impl<'a> LayoutBuilder<'a> {
  pub fn new(layout_ctx: &'a LayoutContext) -> Self {
    Self {
      layout_ctx,
      widgets: vec![],
      constraints: vec![],
    }
  }

  pub fn ctx(&self) -> &'a LayoutContext {
    self.layout_ctx
  }

  pub fn push_widget<W: RawWidget<'a> + 'a>(&mut self, widget: W) {
    let widget: Box<dyn RawWidget<'a> + 'a> = Box::new(widget);
    self.widgets.push(widget);
  }

  pub fn push_constraint(&mut self, prop: Prop<'a>) {
    self.constraints.push(prop);
  }

  pub fn build(self) -> Result<BuildReport<'a>> {
    let z3_ctx = z3::Context::new(&z3::Config::new());
    let mut build_context = Z3BuildContext::new(&z3_ctx);

    let opt = z3::Optimize::new(&z3_ctx);

    let constraints = self
      .widgets
      .iter()
      .map(|x| x.constraints().into_iter())
      .flatten()
      .chain(self.constraints.iter().copied())
      .collect::<Vec<_>>();
    for c in &constraints {
      opt.assert_soft(&c.build_z3(&mut build_context)?, c.weight, None);
    }

    let check_res = opt.check(&[]);
    match check_res {
      z3::SatResult::Sat => {}
      z3::SatResult::Unsat => return Err(LayoutUnsatError::Unsat.into()),
      z3::SatResult::Unknown => return Err(LayoutUnsatError::Unknown.into()),
    }

    let model = opt
      .get_model()
      .expect("check returned sat but failed to get model");
    for w in self.widgets {
      let measures = w.measures();
      let mut refined_values = Vec::with_capacity(measures.len());
      for m in measures {
        let value = model
          .eval(&m.build_z3(&mut build_context)?)
          .expect("check returned sat but model does not provided value for a measure");
        let (num, den) = value
          .as_real()
          .expect("failed to get value from a evaluated Real");
        refined_values.push(num as f64 / den as f64);
      }
      w.paint(&refined_values)?;
    }

    let mut unsatisfied_constraints = vec![];
    let mut satisfied_constraints = vec![];

    for c in &constraints {
      let value = model
        .eval(&c.build_z3(&mut build_context)?)
        .expect("check returned sat but model does not provided value for a prop");
      let value = value
        .as_bool()
        .expect("failed to get value from a evaluated Bool");
      if !value {
        unsatisfied_constraints.push(*c);
      } else {
        satisfied_constraints.push(*c);
      }
    }

    Ok(BuildReport {
      unsatisfied_constraints,
      satisfied_constraints,
    })
  }
}
