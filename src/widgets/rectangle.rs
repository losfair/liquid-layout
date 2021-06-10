use crate::layout::{context::LayoutContext, measure::Measure, prop::Prop, widget::RawWidget};
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RectangleError {
  #[error("empty group")]
  EmptyGroup,
}

#[derive(Copy, Clone)]
pub struct Point<'a> {
  pub x: Measure<'a>,
  pub y: Measure<'a>,
}

pub type RectanglePainter<'a> = Box<dyn FnOnce(RectangleMetrics) -> Result<()> + 'a>;

pub struct Rectangle<'a> {
  pub left: Measure<'a>,
  pub right: Measure<'a>,
  pub top: Measure<'a>,
  pub bottom: Measure<'a>,
  pub width: Measure<'a>,
  pub height: Measure<'a>,

  // `drop` is NOT called on this!
  pub painter: RectanglePainter<'a>,
}

#[derive(Copy, Clone, Debug)]
pub struct RectangleMeasures<'a> {
  pub left: Measure<'a>,
  pub right: Measure<'a>,
  pub top: Measure<'a>,
  pub bottom: Measure<'a>,
  pub width: Measure<'a>,
  pub height: Measure<'a>,
}

#[derive(Debug, Copy, Clone)]
pub struct RectangleMetrics {
  pub left: f64,
  pub right: f64,
  pub top: f64,
  pub bottom: f64,
  pub width: f64,
  pub height: f64,
}

#[allow(dead_code)]
impl<'a> RectangleMeasures<'a> {
  pub fn group_center(group: &[&RectangleMeasures<'a>]) -> Result<Point<'a>> {
    if group.len() == 0 {
      Err(RectangleError::EmptyGroup.into())
    } else {
      let p = group
        .iter()
        .map(|x| (x.left, x.right, x.top, x.bottom))
        .reduce(|(a_l, a_r, a_t, a_b), (b_l, b_r, b_t, b_b)| {
          (a_l.min(b_l), a_r.max(b_r), a_t.min(b_t), a_b.max(b_b))
        })
        .map(|(left, right, top, bottom)| Point {
          x: (left + right) / 2.0,
          y: (top + bottom) / 2.0,
        })
        .unwrap();
      Ok(p)
    }
  }

  pub fn group_leftmost(group: &[&RectangleMeasures<'a>]) -> Result<Measure<'a>> {
    if group.len() == 0 {
      Err(RectangleError::EmptyGroup.into())
    } else {
      let p = group
        .iter()
        .map(|x| x.left)
        .reduce(|a, b| a.min(b))
        .unwrap();
      Ok(p)
    }
  }

  pub fn group_rightmost(group: &[&RectangleMeasures<'a>]) -> Result<Measure<'a>> {
    if group.len() == 0 {
      Err(RectangleError::EmptyGroup.into())
    } else {
      let p = group
        .iter()
        .map(|x| x.right)
        .reduce(|a, b| a.max(b))
        .unwrap();
      Ok(p)
    }
  }

  pub fn group_topmost(group: &[&RectangleMeasures<'a>]) -> Result<Measure<'a>> {
    if group.len() == 0 {
      Err(RectangleError::EmptyGroup.into())
    } else {
      let p = group.iter().map(|x| x.top).reduce(|a, b| a.min(b)).unwrap();
      Ok(p)
    }
  }

  pub fn group_bottommost(group: &[&RectangleMeasures<'a>]) -> Result<Measure<'a>> {
    if group.len() == 0 {
      Err(RectangleError::EmptyGroup.into())
    } else {
      let p = group
        .iter()
        .map(|x| x.bottom)
        .reduce(|a, b| a.max(b))
        .unwrap();
      Ok(p)
    }
  }

  pub fn within(&self, that: &RectangleMeasures<'a>) -> Prop<'a> {
    self.left_to(that.right, 0.0)
      & self.right_to(that.left, 0.0)
      & self.top_to(that.bottom, 0.0)
      & self.bottom_to(that.top, 0.0)
  }

  pub fn center(&self) -> Result<Point<'a>> {
    Self::group_center(&[self])
  }

  pub fn left_to(&self, that: Measure<'a>, distance: f64) -> Prop<'a> {
    self.right.prop_eq(that - distance)
  }

  pub fn right_to(&self, that: Measure<'a>, distance: f64) -> Prop<'a> {
    self.left.prop_eq(that + distance)
  }

  pub fn top_to(&self, that: Measure<'a>, distance: f64) -> Prop<'a> {
    self.bottom.prop_eq(that - distance)
  }

  pub fn bottom_to(&self, that: Measure<'a>, distance: f64) -> Prop<'a> {
    self.top.prop_eq(that + distance)
  }
}

#[allow(dead_code)]
impl<'a> Rectangle<'a> {
  pub fn measures(&self) -> RectangleMeasures<'a> {
    RectangleMeasures {
      left: self.left,
      right: self.right,
      top: self.top,
      bottom: self.bottom,
      width: self.width,
      height: self.height,
    }
  }

  pub fn square(ctx: &'a LayoutContext, painter: RectanglePainter<'a>) -> Self {
    let border_length = Measure::new_unbound(ctx);

    Self {
      left: Measure::new_unbound(ctx),
      right: Measure::new_unbound(ctx),
      top: Measure::new_unbound(ctx),
      bottom: Measure::new_unbound(ctx),
      width: border_length,
      height: border_length,
      painter,
    }
  }

  pub fn row_spacer(ctx: &'a LayoutContext, flex_unit: Measure<'a>) -> Self {
    Self {
      left: Measure::new_unbound(ctx),
      right: Measure::new_unbound(ctx),
      top: Measure::new_unbound(ctx),
      bottom: Measure::new_unbound(ctx),
      width: flex_unit,
      height: Measure::new_unbound(ctx),
      painter: Box::new(|metrics| {
        log::debug!("row_spacer metrics: {:?}", metrics);
        Ok(())
      }),
    }
  }

  pub fn unbound(ctx: &'a LayoutContext, painter: RectanglePainter<'a>) -> Self {
    Self {
      left: Measure::new_unbound(ctx),
      right: Measure::new_unbound(ctx),
      top: Measure::new_unbound(ctx),
      bottom: Measure::new_unbound(ctx),
      width: Measure::new_unbound(ctx),
      height: Measure::new_unbound(ctx),
      painter,
    }
  }

  pub fn with_width_and_height(
    ctx: &'a LayoutContext,
    width: f64,
    height: f64,
    painter: RectanglePainter<'a>,
  ) -> Self {
    Self {
      left: Measure::new_unbound(ctx),
      right: Measure::new_unbound(ctx),
      top: Measure::new_unbound(ctx),
      bottom: Measure::new_unbound(ctx),
      width: Measure::new_const(ctx, width).unwrap(),
      height: Measure::new_const(ctx, height).unwrap(),
      painter,
    }
  }
}

impl<'a> RawWidget<'a> for Rectangle<'a> {
  fn measures(&self) -> Vec<Measure<'a>> {
    vec![
      self.left,
      self.right,
      self.top,
      self.bottom,
      self.width,
      self.height,
    ]
  }

  fn constraints(&self) -> Vec<Prop<'a>> {
    vec![
      (self.left + self.width).prop_eq(self.right),
      (self.top + self.height).prop_eq(self.bottom),
      self.top.prop_ge(Measure::zero(self.top.ctx)),
      self.left.prop_ge(Measure::zero(self.left.ctx)),
      self
        .width
        .prop_ge(Measure::new_const(self.width.ctx, 0.0).unwrap()),
      self
        .height
        .prop_ge(Measure::new_const(self.height.ctx, 0.0).unwrap()),
    ]
  }

  fn paint(self: Box<Self>, measures: &[f64]) -> Result<()> {
    let metrics = RectangleMetrics {
      left: measures[0],
      right: measures[1],
      top: measures[2],
      bottom: measures[3],
      width: measures[4],
      height: measures[5],
    };

    (self.painter)(metrics)
  }
}
