extern crate gl;
extern crate sdl2;

use std::time::Instant;

pub mod render_gl;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);

    let window = video_subsystem.window("Game", 900, 700).opengl().resizable().build().unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let _gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    // set up shader program
    use std::ffi::CString;
    let vert_shader = render_gl::Shader::from_vert_source(&CString::new(include_str!("triangle.vert")).unwrap()).unwrap();
    let frag_shader = render_gl::Shader::from_frag_source(&CString::new(include_str!("triangle.frag")).unwrap()).unwrap();
    let shader_program = render_gl::Program::from_shaders(&[vert_shader, frag_shader]).unwrap();

    let mut empty_vao: gl::types::GLuint = 0;
    unsafe {
        gl::Viewport(0, 0, 900, 700);
        gl::ClearColor(0.3, 0.3, 0.5, 1.0);

        gl::GenVertexArrays(1, &mut empty_vao);
    }

    let last_time = Instant::now();

    // main loop
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        let elapsed = last_time.elapsed();

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // draw triangle

        shader_program.set_used();
        unsafe {
            gl::Uniform2f(2, 900.0, 700.0); //Screen Resolution
            gl::Uniform3f(3, 0.0, 0.0, 0.0); //Camera Position
            gl::Uniform1f(4, elapsed.as_secs() as f32 + (elapsed.subsec_millis() as f32 / 1_000f32)); //Time

            gl::BindVertexArray(empty_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
        window.gl_swap_window();
    }
}