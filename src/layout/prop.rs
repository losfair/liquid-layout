use anyhow::Result;
use std::fmt::{Debug, Display};
use std::ops::{BitAnd, BitOr, Not};
use z3::ast::{Ast, Bool};

use super::measure::MeasureVariant;
use super::{
  context::{LayoutContext, Z3BuildContext},
  measure::Measure,
};

/// A proposition on measurements or other propositions.
#[derive(Copy, Clone)]
pub struct Prop<'a> {
  pub ctx: &'a LayoutContext,
  pub(super) variant: &'a PropVariant<'a>,
  pub(super) weight: u32,
}

impl<'a> Debug for Prop<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Prop({}) {{ {:?} }}", self.weight, self.variant)
  }
}

#[derive(Copy, Clone, Debug)]
pub enum PropVariant<'a> {
  Eq(Measure<'a>, Measure<'a>),
  Lt(Measure<'a>, Measure<'a>),
  Le(Measure<'a>, Measure<'a>),
  Gt(Measure<'a>, Measure<'a>),
  Ge(Measure<'a>, Measure<'a>),
  Or(Prop<'a>, Prop<'a>),
  And(Prop<'a>, Prop<'a>),
  Not(Prop<'a>),
}

#[allow(dead_code)]
impl<'a> Prop<'a> {
  pub fn with_weight(mut self, weight: u32) -> Self {
    self.weight = weight;
    self
  }

  pub fn select(self, left: Measure<'a>, right: Measure<'a>) -> Measure<'a> {
    Measure {
      ctx: self.ctx,
      variant: self
        .ctx
        .alloc
        .alloc(MeasureVariant::Select(self, left, right)),
    }
  }

  pub fn build_z3<'ctx>(self, build_ctx: &mut Z3BuildContext<'ctx>) -> Result<Bool<'ctx>> {
    let key = self.variant as *const _ as usize;
    if let Some(x) = build_ctx.prop_cache.get(&key) {
      return Ok(x.clone());
    }
    let res = self.do_build_z3(build_ctx)?;
    build_ctx.prop_cache.insert(key, res.clone());
    Ok(res)
  }

  fn do_build_z3<'ctx>(self, build_ctx: &mut Z3BuildContext<'ctx>) -> Result<Bool<'ctx>> {
    use PropVariant as V;
    let z3_ctx = build_ctx.z3_ctx;
    Ok(match *self.variant {
      V::Eq(left, right) => left.build_z3(build_ctx)?._eq(&right.build_z3(build_ctx)?),
      V::Lt(left, right) => left.build_z3(build_ctx)?.lt(&right.build_z3(build_ctx)?),
      V::Le(left, right) => left.build_z3(build_ctx)?.le(&right.build_z3(build_ctx)?),
      V::Gt(left, right) => left.build_z3(build_ctx)?.gt(&right.build_z3(build_ctx)?),
      V::Ge(left, right) => left.build_z3(build_ctx)?.ge(&right.build_z3(build_ctx)?),
      V::Or(left, right) => Bool::or(
        z3_ctx,
        &[&left.build_z3(build_ctx)?, &right.build_z3(build_ctx)?],
      ),
      V::And(left, right) => Bool::and(
        z3_ctx,
        &[&left.build_z3(build_ctx)?, &right.build_z3(build_ctx)?],
      ),
      V::Not(that) => that.build_z3(build_ctx)?.not(),
    })
  }
}

impl<'a> Display for Prop<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.variant {
      PropVariant::Eq(l, r) => write!(f, "{} == {}", l, r),
      PropVariant::Lt(l, r) => write!(f, "{} < {}", l, r),
      PropVariant::Le(l, r) => write!(f, "{} <= {}", l, r),
      PropVariant::Gt(l, r) => write!(f, "{} > {}", l, r),
      PropVariant::Ge(l, r) => write!(f, "{} >= {}", l, r),
      PropVariant::Or(l, r) => write!(f, "({}) or ({})", l, r),
      PropVariant::And(l, r) => write!(f, "({}) and ({})", l, r),
      PropVariant::Not(x) => write!(f, "not ({})", x),
    }
  }
}

impl<'a> BitOr for Prop<'a> {
  type Output = Self;
  fn bitor(self, that: Prop<'a>) -> Self {
    Self {
      ctx: self.ctx,
      variant: self.ctx.alloc.alloc(PropVariant::Or(self, that)),
      weight: 10,
    }
  }
}

impl<'a> BitAnd for Prop<'a> {
  type Output = Self;
  fn bitand(self, that: Prop<'a>) -> Self {
    Self {
      ctx: self.ctx,
      variant: self.ctx.alloc.alloc(PropVariant::And(self, that)),
      weight: 10,
    }
  }
}

impl<'a> Not for Prop<'a> {
  type Output = Self;
  fn not(self) -> Self {
    Self {
      ctx: self.ctx,
      variant: self.ctx.alloc.alloc(PropVariant::Not(self)),
      weight: 10,
    }
  }
}
