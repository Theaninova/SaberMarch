extern crate sdl2;
extern crate gl;

use std::ffi::{CString, CStr};

pub mod render_gl;

fn main() {
    //Specify the minimum required OpenGL Version
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 5);

    //Create a window
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let window = video_subsystem.window("SaberMarch", 900, 700).opengl().resizable().build().unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let _gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    let mut event_pump = sdl.event_pump().unwrap();

    'main: loop {
        unsafe {
            gl::Viewport(0, 0, 900, 700);
            gl::ClearColor(0.3, 0.3, 0.5, 1.0);
        }

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main, _ => {},
            }
        }

        window.gl_swap_window();
    }
}