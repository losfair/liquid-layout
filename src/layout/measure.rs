use std::{
  f64::EPSILON,
  fmt::Display,
  ops::{Add, Div, Mul, Sub},
};

use anyhow::Result;
use fraction::GenericFraction;
use thiserror::Error;
use z3::ast::Real;

use super::{
  context::{LayoutContext, Z3BuildContext},
  prop::{Prop, PropVariant},
};
use std::fmt::Debug;

/// A real-number measurement of a property of an object.
#[derive(Copy, Clone)]
pub struct Measure<'a> {
  pub ctx: &'a LayoutContext,
  pub(super) variant: &'a MeasureVariant<'a>,
}

impl<'a> Debug for Measure<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Measure {{ {:?} }}", self.variant)
  }
}

#[derive(Error, Debug)]
pub enum MeasureError {
  #[error("bad const")]
  BadConst,
}

#[derive(Copy, Clone, Debug)]
pub enum MeasureVariant<'a> {
  Unbound,
  Const(i32, i32),
  Add(Measure<'a>, Measure<'a>),
  Sub(Measure<'a>, Measure<'a>),
  Mul(Measure<'a>, Measure<'a>),
  Div(Measure<'a>, Measure<'a>),
  Select(Prop<'a>, Measure<'a>, Measure<'a>),
}

struct UnsafelyAssumeThreadSafe<T>(T);
unsafe impl<T> Send for UnsafelyAssumeThreadSafe<T> {}
unsafe impl<T> Sync for UnsafelyAssumeThreadSafe<T> {}

static SMALL_MEASURE_CONSTS: UnsafelyAssumeThreadSafe<[MeasureVariant<'static>; 16]> =
  UnsafelyAssumeThreadSafe([
    MeasureVariant::Const(0, 1),
    MeasureVariant::Const(1, 1),
    MeasureVariant::Const(2, 1),
    MeasureVariant::Const(3, 1),
    MeasureVariant::Const(4, 1),
    MeasureVariant::Const(5, 1),
    MeasureVariant::Const(6, 1),
    MeasureVariant::Const(7, 1),
    MeasureVariant::Const(8, 1),
    MeasureVariant::Const(9, 1),
    MeasureVariant::Const(10, 1),
    MeasureVariant::Const(11, 1),
    MeasureVariant::Const(12, 1),
    MeasureVariant::Const(13, 1),
    MeasureVariant::Const(14, 1),
    MeasureVariant::Const(15, 1),
  ]);

#[allow(dead_code)]
impl<'a> Measure<'a> {
  pub fn zero(ctx: &'a LayoutContext) -> Self {
    Measure {
      ctx,
      variant: &SMALL_MEASURE_CONSTS.0[0],
    }
  }

  pub fn new_const(ctx: &'a LayoutContext, value: f64) -> Result<Self, MeasureError> {
    let value = ((value * 100.0) as i64) as f64 / 100.0;

    // Small integer pool
    if ((value as i64) as f64 - value).abs() < EPSILON {
      let candidates = &SMALL_MEASURE_CONSTS.0;
      let index = value as i64;
      if index >= 0 && index < candidates.len() as i64 {
        return Ok(Measure {
          ctx,
          variant: &candidates[index as usize],
        });
      }
    }

    let frac = GenericFraction::<i32>::from(value);
    let sign: i32 = if value < 0.0 { -1 } else { 1 };
    Ok(Measure {
      ctx,
      variant: ctx.alloc.alloc(MeasureVariant::Const(
        *frac.numer().ok_or_else(|| MeasureError::BadConst)? * sign,
        *frac.denom().ok_or_else(|| MeasureError::BadConst)?,
      )),
    })
  }

  pub fn new_unbound(ctx: &'a LayoutContext) -> Self {
    Measure {
      ctx,
      variant: ctx.alloc.alloc(MeasureVariant::Unbound),
    }
  }

  pub fn is_unbound(&self) -> bool {
    match self.variant {
      &MeasureVariant::Unbound => true,
      _ => false,
    }
  }

  pub fn build_z3<'ctx>(self, build_ctx: &mut Z3BuildContext<'ctx>) -> Result<Real<'ctx>> {
    let key = self.variant as *const _ as usize;
    if let Some(x) = build_ctx.measure_cache.get(&key) {
      return Ok(x.clone());
    }
    let res = self.do_build_z3(build_ctx)?;
    build_ctx.measure_cache.insert(key, res.clone());
    Ok(res)
  }

  fn do_build_z3<'ctx>(self, build_ctx: &mut Z3BuildContext<'ctx>) -> Result<Real<'ctx>> {
    use MeasureVariant as V;
    let z3_ctx = build_ctx.z3_ctx;
    Ok(match *self.variant {
      V::Unbound => Real::fresh_const(z3_ctx, "measure_"),
      V::Const(num, den) => Real::from_real(z3_ctx, num, den),
      V::Add(left, right) => left.build_z3(build_ctx)?.add(right.build_z3(build_ctx)?),
      V::Sub(left, right) => left.build_z3(build_ctx)?.sub(right.build_z3(build_ctx)?),
      V::Mul(left, right) => left.build_z3(build_ctx)?.mul(right.build_z3(build_ctx)?),
      V::Div(left, right) => left.build_z3(build_ctx)?.div(right.build_z3(build_ctx)?),
      V::Select(condition, left, right) => condition
        .build_z3(build_ctx)?
        .ite(&left.build_z3(build_ctx)?, &right.build_z3(build_ctx)?),
    })
  }

  pub fn prop_eq(self, that: Self) -> Prop<'a> {
    Prop {
      ctx: self.ctx,
      variant: self.ctx.alloc.alloc(PropVariant::Eq(self, that)),
      weight: 10,
    }
  }

  pub fn prop_lt(self, that: Self) -> Prop<'a> {
    Prop {
      ctx: self.ctx,
      variant: self.ctx.alloc.alloc(PropVariant::Lt(self, that)),
      weight: 10,
    }
  }

  pub fn prop_le(self, that: Self) -> Prop<'a> {
    Prop {
      ctx: self.ctx,
      variant: self.ctx.alloc.alloc(PropVariant::Le(self, that)),
      weight: 10,
    }
  }

  pub fn prop_gt(self, that: Self) -> Prop<'a> {
    Prop {
      ctx: self.ctx,
      variant: self.ctx.alloc.alloc(PropVariant::Gt(self, that)),
      weight: 10,
    }
  }

  pub fn prop_ge(self, that: Self) -> Prop<'a> {
    Prop {
      ctx: self.ctx,
      variant: self.ctx.alloc.alloc(PropVariant::Ge(self, that)),
      weight: 10,
    }
  }

  pub fn min(self, that: Self) -> Measure<'a> {
    self.prop_lt(that).select(self, that)
  }

  pub fn max(self, that: Self) -> Measure<'a> {
    self.prop_gt(that).select(self, that)
  }
}

impl<'a> Display for Measure<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.variant {
      MeasureVariant::Unbound => write!(f, "<{:p}>", self.variant),
      MeasureVariant::Const(num, den) => write!(f, "{}", *num as f64 / *den as f64),
      MeasureVariant::Add(l, r)
        if r.variant as *const _ == &SMALL_MEASURE_CONSTS.0[0] as *const _ =>
      {
        write!(f, "{}", l)
      }
      MeasureVariant::Add(l, r) => write!(f, "({} + {})", l, r),
      MeasureVariant::Sub(l, r)
        if r.variant as *const _ == &SMALL_MEASURE_CONSTS.0[0] as *const _ =>
      {
        write!(f, "{}", l)
      }
      MeasureVariant::Sub(l, r) => write!(f, "({} - {})", l, r),
      MeasureVariant::Mul(l, r) => write!(f, "({} * {})", l, r),
      MeasureVariant::Div(l, r) => write!(f, "({} / {})", l, r),
      MeasureVariant::Select(cond, l, r) => match cond.variant {
        PropVariant::Lt(cond_l, cond_r)
          if cond_l.variant as *const _ == l.variant as *const _
            && cond_r.variant as *const _ == r.variant as *const _ =>
        {
          write!(f, "(min {} {})", l, r)
        }
        PropVariant::Gt(cond_l, cond_r)
          if cond_l.variant as *const _ == l.variant as *const _
            && cond_r.variant as *const _ == r.variant as *const _ =>
        {
          write!(f, "(max {} {})", l, r)
        }
        _ => write!(f, "(select ({}) {} {})", cond, l, r),
      },
    }
  }
}

impl<'a> Add for Measure<'a> {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    Self {
      ctx: self.ctx,
      variant: self.ctx.alloc.alloc(MeasureVariant::Add(self, other)),
    }
  }
}

impl<'a> Add<f64> for Measure<'a> {
  type Output = Self;

  fn add(self, other: f64) -> Self {
    self + Measure::new_const(self.ctx, other).unwrap()
  }
}

impl<'a> Sub for Measure<'a> {
  type Output = Self;

  fn sub(self, other: Self) -> Self {
    Self {
      ctx: self.ctx,
      variant: self.ctx.alloc.alloc(MeasureVariant::Sub(self, other)),
    }
  }
}

impl<'a> Sub<f64> for Measure<'a> {
  type Output = Self;

  fn sub(self, other: f64) -> Self {
    self - Measure::new_const(self.ctx, other).unwrap()
  }
}

impl<'a> Mul for Measure<'a> {
  type Output = Self;

  fn mul(self, other: Self) -> Self {
    Self {
      ctx: self.ctx,
      variant: self.ctx.alloc.alloc(MeasureVariant::Mul(self, other)),
    }
  }
}

impl<'a> Mul<f64> for Measure<'a> {
  type Output = Self;

  fn mul(self, other: f64) -> Self {
    self * Measure::new_const(self.ctx, other).unwrap()
  }
}

impl<'a> Div for Measure<'a> {
  type Output = Self;

  fn div(self, other: Self) -> Self {
    Self {
      ctx: self.ctx,
      variant: self.ctx.alloc.alloc(MeasureVariant::Div(self, other)),
    }
  }
}

impl<'a> Div<f64> for Measure<'a> {
  type Output = Self;

  fn div(self, other: f64) -> Self {
    self / Measure::new_const(self.ctx, other).unwrap()
  }
}
