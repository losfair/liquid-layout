use rand::Rng;

use super::Rectangle;
use crate::layout::{builder::LayoutBuilder, context::LayoutContext, measure::Measure};

#[test]
fn test_rectangle_success() {
  let ctx = LayoutContext::new();
  let mut builder = LayoutBuilder::new(&ctx);

  let rect = Rectangle::with_width_and_height(
    &ctx,
    5.0,
    10.0,
    Box::new(|metrics| {
      println!("{:?}", metrics);
      Ok(())
    }),
  );
  builder.push_widget(rect);

  let report = builder.build().unwrap();
  assert!(report.unsatisfied_constraints.is_empty());
}

#[test]
fn test_rectangle_fail() {
  let ctx = LayoutContext::new();
  let mut builder = LayoutBuilder::new(&ctx);

  let mut rect = Rectangle::with_width_and_height(&ctx, 5.0, 10.0, Box::new(|_| Ok(())));
  rect.top = Measure::new_const(&ctx, 0.0).unwrap();
  rect.bottom = Measure::new_const(&ctx, 1.0).unwrap();
  builder.push_widget(rect);

  let report = builder.build().unwrap();
  assert!(!report.unsatisfied_constraints.is_empty());
  println!("test_rectangle_fail: {:?}", report);
}

#[test]
fn test_many_rectangles() {
  let ctx = LayoutContext::new();
  let mut builder = LayoutBuilder::new(&ctx);
  let mut rng = rand::thread_rng();

  for _ in 0..10000 {
    let width: f64 = rng.gen_range(10.0, 100.0);
    let height: f64 = rng.gen_range(10.0, 100.0);
    let top: f64 = rng.gen_range(10.0, 100.0);
    let left: f64 = rng.gen_range(10.0, 100.0);
    let mut rect = Rectangle::with_width_and_height(&ctx, width, height, Box::new(|_| Ok(())));
    rect.top = Measure::new_const(&ctx, top).unwrap();
    rect.left = Measure::new_const(&ctx, left).unwrap();
    builder.push_widget(rect);
  }

  let report = builder.build().unwrap();
  assert!(report.unsatisfied_constraints.is_empty());
}

#[test]
fn test_nesting_rectangles() {
  let ctx = LayoutContext::new();
  let mut builder = LayoutBuilder::new(&ctx);

  let mut last_frame = (
    Measure::new_const(&ctx, -100.0).unwrap(),
    Measure::new_const(&ctx, 100.0).unwrap(),
    Measure::new_const(&ctx, -100.0).unwrap(),
    Measure::new_const(&ctx, 100.0).unwrap(),
  );

  for _ in 0..100 {
    let rect = Rectangle::unbound(&ctx, Box::new(|_| Ok(())));
    builder.push_constraint(rect.left.prop_gt(last_frame.0));
    builder.push_constraint(rect.right.prop_lt(last_frame.1));
    builder.push_constraint(rect.top.prop_gt(last_frame.2));
    builder.push_constraint(rect.bottom.prop_lt(last_frame.3));
    builder.push_constraint(rect.width.prop_gt(Measure::new_const(&ctx, 3.0).unwrap()));
    builder.push_constraint(rect.height.prop_gt(Measure::new_const(&ctx, 3.0).unwrap()));
    last_frame.0 = rect.left;
    last_frame.1 = rect.right;
    last_frame.2 = rect.top;
    last_frame.3 = rect.bottom;
    builder.push_widget(rect);
  }

  let report = builder.build().unwrap();
  assert!(report.unsatisfied_constraints.is_empty());
}
