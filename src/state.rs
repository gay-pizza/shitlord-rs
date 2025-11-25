use crate::renderer::Renderer;

pub(crate) mod beatoburnerstate;
pub(crate) mod splashstate;
pub(crate) mod gamestate;

pub(crate) trait State {
  fn init(&mut self) {}
  fn quit(&mut self) {}
  fn tick(&mut self, deltatime: f32) -> StateCmd;
  fn load(&mut self, renderer: &mut Renderer);
  fn draw(&mut self, renderer: &mut Renderer, deltatime: f32);
}

#[allow(dead_code)]
pub(crate) enum StateCmd {
  ChangeState(Box<dyn State>),
  Quit,
  Continue,
}
