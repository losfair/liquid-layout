use std::collections::HashMap;

use bumpalo::Bump;
use z3::ast::{Bool, Real};

pub struct LayoutContext {
  pub alloc: Bump,
}

impl LayoutContext {
  pub fn new() -> Self {
    LayoutContext { alloc: Bump::new() }
  }
}

pub struct Z3BuildContext<'ctx> {
  pub prop_cache: HashMap<usize, Bool<'ctx>>,
  pub measure_cache: HashMap<usize, Real<'ctx>>,
  pub z3_ctx: &'ctx z3::Context,
}

impl<'ctx> Z3BuildContext<'ctx> {
  pub fn new(z3_ctx: &'ctx z3::Context) -> Self {
    Self {
      prop_cache: HashMap::new(),
      measure_cache: HashMap::new(),
      z3_ctx,
    }
  }
}
