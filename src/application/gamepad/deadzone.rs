use crate::maths::vector2::Vec2f;

#[allow(dead_code)]
pub(crate) trait DeadZone {
  fn axis_deadzone(&self, min: Self, max: Self) -> Self;
}

impl DeadZone for f32 {
  fn axis_deadzone(&self, min: Self, max: Self) -> f32 {
    let range = max - min;
    assert_ne!(range, 0.0);
    let abs = self.abs();
    if abs <= min {
      0f32
    } else if abs >= max {
      self.signum()
    } else {
      (abs - min).copysign(*self) / range
    }
  }
}

#[allow(dead_code)]
pub(crate) trait DeadZone2D<T> {
  fn cardinal_deadzone(&self, min: T, max: T) -> Self;
  fn radial_deadzone(&self, min: T, max: T) -> Self;
}

impl DeadZone2D<f32> for Vec2f {
  #[inline]
  fn cardinal_deadzone(&self, min: f32, max: f32) -> Vec2f {
    Vec2f::new(self.x.axis_deadzone(min, max), self.y.axis_deadzone(min, max))
  }

  fn radial_deadzone(&self, min: f32, max: f32) -> Vec2f {
    let range = max - min;
    assert_ne!(range, 0f32);
    let magnitude = self.mag();
    if magnitude < f32::EPSILON || magnitude < min {
      Vec2f::ZERO
    } else if magnitude >= max {
      *self / magnitude
    } else {
      let rescale = (magnitude - min) / range;
      *self / magnitude * rescale
    }
  }
}
