mod actor;
mod application;
mod maths;
mod renderer;
mod state;
mod random;
mod fpscalculator;

use crate::application::Application;
use sdl3_sys::events::SDL_Event;
use sdl3_sys::init::{SDL_AppResult, SDL_APP_CONTINUE, SDL_APP_FAILURE};
use sdl3_sys::main::{SDL_EnterAppMainCallbacks, SDL_RunApp};
use std::ffi::CString;
use std::iter::once;
use std::ptr::null_mut;

pub fn main() {
  let cstr_args = std::env::args()
    .map(|arg| CString::new(arg).unwrap())
    .collect::<Vec<CString>>();
  let mut argv = cstr_args.iter()
    .map(|arg| arg.as_ptr() as *mut core::ffi::c_char)
    .chain(once(null_mut() as *mut core::ffi::c_char))
    .collect::<Vec<*mut core::ffi::c_char>>();

  unsafe {
    std::process::exit(SDL_RunApp(cstr_args.len() as core::ffi::c_int, argv.as_mut_ptr(),
      Some(SDL_main), null_mut()));
  }
}

#[allow(non_snake_case)]
extern "C" fn SDL_main(argc: core::ffi::c_int, argv: *mut *mut ::core::ffi::c_char
) -> core::ffi::c_int {
  unsafe {
    SDL_EnterAppMainCallbacks(argc, argv,
      Some(SDL_AppInit), Some(SDL_AppIterate), Some(SDL_AppEvent), Some(SDL_AppQuit))
  }
}

#[allow(non_snake_case)]
extern "C" fn SDL_AppInit(appstate: *mut *mut core::ffi::c_void,
  _argc: core::ffi::c_int, _argv: *mut *mut core::ffi::c_char
) -> SDL_AppResult {
  let application = Box::into_raw(Box::new(Application::new()));
  unsafe {
    *appstate = application as *mut core::ffi::c_void;
    match application.as_mut().expect("application was null").init() {
      Ok(_) => SDL_APP_CONTINUE,
      Err(err) => {
        eprintln!("ERROR: {:?}", err);
        SDL_APP_FAILURE
      }
    }
  }
}

#[allow(non_snake_case)]
extern "C" fn SDL_AppIterate(appstate: *mut core::ffi::c_void) -> SDL_AppResult {
  let application = unsafe { (appstate as *mut Application).as_mut() }
    .expect("appstate was null");
  application.iterate()
}

#[allow(non_snake_case)]
extern "C" fn SDL_AppEvent(appstate: *mut core::ffi::c_void, event: *mut SDL_Event
) -> SDL_AppResult {
  unsafe {
    let application = (appstate as *mut Application).as_mut()
      .expect("appstate was null");
    application.event(*event)
  }
}

#[allow(non_snake_case)]
extern "C" fn SDL_AppQuit(appstate: *mut core::ffi::c_void, _result: SDL_AppResult) {
  let mut application = unsafe { Box::from_raw(appstate as *mut Application) };
  application.quit();
  drop(application);
}
