use crate::application::timehelper::TimeHelper;
use crate::fpscalculator::FPSCalculator;
use crate::application::gamepad::GamePad;
use crate::maths::vector2::Vec2f;
use crate::renderer::colour::Colour;
use crate::renderer::Renderer;
use crate::state::splashstate::SplashState;
use crate::state::{State, StateCmd};
use sdl3_sys::everything::*;
use std::ffi::{c_int, CStr, CString};
use std::ptr::addr_of_mut;
use std::ptr::null_mut;
use crate::application::keyboard::Keyboard;

pub(crate) mod gamepad;
mod timehelper;
pub(crate) mod keyboard;

pub(crate) struct Application {
  window: *mut SDL_Window,
  renderer: Option<Renderer>,
  state: Option<Box<dyn State>>,
  time: TimeHelper,
  fps_counter: FPSCalculator,
  fps_string: CString,
  currently_fullscreen: bool,
}

impl Application {
  pub(crate) fn new() -> Application {
    Application {
      window: null_mut(),
      renderer: None,
      state: None,
      time: Default::default(),
      fps_counter: Default::default(),
      fps_string: CString::new("").unwrap(),
      currently_fullscreen: false,
    }
  }

  fn change_state<T: State + Default + 'static>(&mut self) {
    if let Some(mut state) = self.state.take() {
      state.quit();
    }
    self.state = Some(Box::new(T::default()));
    if let Some(state) = self.state.as_mut() {
      state.load(self.renderer.as_mut().unwrap());
      state.init();
    }
  }

  pub(crate) fn init(&mut self) -> Result<(), AppError> {
    unsafe {
      SDL_SetCurrentThreadPriority(SDL_THREAD_PRIORITY_HIGH);
      if !SDL_Init(SDL_INIT_VIDEO | SDL_INIT_GAMEPAD) {
        return Err(AppError::Error(format!("SDL_Init failed: {}",
          CStr::from_ptr(SDL_GetError()).to_string_lossy())));
      }

      let wintitle = CString::new("Find the computer room!").unwrap();
      let winflags = SDL_WINDOW_HIGH_PIXEL_DENSITY | SDL_WINDOW_RESIZABLE;
      self.window = SDL_CreateWindow(wintitle.as_ptr(), 640, 480, winflags);
      if self.window.is_null() {
        return Err(AppError::Error(format!("SDL_CreateWindow failed: {}",
          CStr::from_ptr(SDL_GetError()).to_string_lossy())));
      }
    }

    self.renderer = Some(Renderer::new(self.window, 640, 480, true)?);
    self.change_state::<SplashState>();
    self.time.init();

    Ok(())
  }

  #[allow(unsafe_op_in_unsafe_fn)]
  pub(crate) unsafe fn event(&mut self, event: SDL_Event) -> SDL_AppResult {
    match SDL_EventType(event.r#type) {
      SDL_EVENT_QUIT => SDL_APP_SUCCESS,
      SDL_EVENT_KEY_DOWN | SDL_EVENT_KEY_UP => match event.key.key {
        SDLK_ESCAPE => SDL_APP_SUCCESS,
        SDLK_RETURN if event.key.down && !event.key.repeat && event.key.r#mod & SDL_KMOD_ALT != 0 => {
          SDL_SetWindowFullscreen(self.window, !self.currently_fullscreen);
          SDL_APP_CONTINUE
        }
        _ => {
          Keyboard::key_event(event.key.scancode, event.key.down, event.key.repeat);
          SDL_APP_CONTINUE
        }
      }
      SDL_EVENT_WINDOW_ENTER_FULLSCREEN => {
        self.currently_fullscreen = true;
        SDL_APP_CONTINUE
      }
      SDL_EVENT_WINDOW_LEAVE_FULLSCREEN => {
        self.currently_fullscreen = false;
        SDL_APP_CONTINUE
      }
      SDL_EVENT_GAMEPAD_ADDED => {
        GamePad::connected_event(event.gdevice.which);
        SDL_APP_CONTINUE
      }
      SDL_EVENT_GAMEPAD_REMOVED => {
        GamePad::removed_event(event.gdevice.which);
        SDL_APP_CONTINUE
      }
      SDL_EVENT_GAMEPAD_BUTTON_DOWN | SDL_EVENT_GAMEPAD_BUTTON_UP => {
        GamePad::button_event(event.gbutton.which, SDL_GamepadButton(event.gbutton.button as c_int), event.gbutton.down);
        SDL_APP_CONTINUE
      }
      SDL_EVENT_GAMEPAD_AXIS_MOTION => {
        GamePad::axis_event(event.gaxis.which, SDL_GamepadAxis(event.gaxis.axis as c_int), event.gaxis.value);
        SDL_APP_CONTINUE
      }
      _ => SDL_APP_CONTINUE
    }
  }

  fn draw(&mut self, deltatime: f32) {
    if let Some(ref mut renderer) = self.renderer {
      if let Some(ref mut state) = self.state {
        state.draw(renderer, deltatime);
      }
      renderer.set_draw_colour(Colour::hex(0xFF1F4FFF));
      renderer.text_cstr(Vec2f::ONE * 5.0, &self.fps_string);
      renderer.present()
    }
  }

  pub(crate) fn quit(&mut self) {
    if let Some(ref mut state) = self.state {
      state.quit();
    }
    self.state = None;
    self.renderer = None;
    unsafe {
      SDL_DestroyWindow(self.window);
      SDL_Quit();
    }
  }

  pub(crate) fn iterate(&mut self) -> SDL_AppResult {
    self.time.frame_advance();

    // Update FPS metrics
    self.fps_counter.frame(self.time.get_duration(),
      |s| self.fps_string = CString::new(format!("FPS: {}", s)).unwrap());

    // Poll events
    unsafe {
      let mut event: SDL_Event = std::mem::zeroed();
      while SDL_PollEvent(addr_of_mut!(event)) {
        self.event(event);
      }
    }

    // Calculate delta time
    const MIN_DELTA_TIME: f32 = 1.0 / 15.0;
    let delta = f32::min(MIN_DELTA_TIME, self.time.get_deltatime() as f32);

    // Tick and draw
    let cmd = self.state.as_mut().map_or(StateCmd::Continue, |state| state.tick(delta));
    self.draw(delta);

    match cmd {
      // Scene asked us to switch to a different one
      StateCmd::ChangeState(new_state) => {
        if let Some(mut state) = self.state.take() {
          state.quit();
        }
        self.state = Some(new_state);
        if let Some(ref mut new_state) = self.state {
          new_state.load(self.renderer.as_mut().unwrap());
          new_state.init();
        }
      }
      // Scene returned quit
      StateCmd::Quit => {
        return SDL_APP_SUCCESS
      }
      // No command
      StateCmd::Continue => {}
    }

    GamePad::advance_frame();
    Keyboard::advance_frame();

    SDL_APP_CONTINUE
  }
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum AppError {
  Error(String),
  IOError(std::io::Error),
}

impl From<std::io::Error> for AppError {
  fn from(err: std::io::Error) -> Self { AppError::IOError(err) }
}
