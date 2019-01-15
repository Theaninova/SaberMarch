extern crate gl;
extern crate sdl2;
extern crate openvr;

use std::time::Instant;

pub mod render_gl;

fn main() {
    let context = match unsafe { openvr::init(openvr::ApplicationType::Scene) } {
        Ok(ivr) => ivr,
        Err(err) => {
            println!("Failed to initialize OpenVR: {}", err);
            return;
        }
    };
    print!("OpenVR was initialized Successfully.");

    let system = match context.system() {
        Ok(sys) => sys,
        Err(err) => {
            println!("Failed to get system interface: {}", err);
            return;
        }
    };

    let comp = match context.compositor() {
        Ok(ext) => ext,
        Err(err) => {
            println!("Failed to create IVRCompositor subsystem: {}", err);
            return;
        }
    };

    println!("Left Eye Matrix: {:?}", system.eye_to_head_transform(openvr::Eye::Left));
    println!("Left Eye Matrix: {:?}", system.eye_to_head_transform(openvr::Eye::Right));
    println!("\tRecommended size: {:?}", system.recommended_render_target_size());
    println!("\tVsync: {:?}", system.time_since_last_vsync());

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);

    let window = video_subsystem.window("OpenSabers", 1024, 768).opengl().resizable().build().unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let _gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    // set up shader program
    use std::ffi::CString;
    let vert_shader = render_gl::Shader::from_vert_source(&CString::new(include_str!("triangle.vert")).unwrap()).unwrap();
    let frag_shader = render_gl::Shader::from_frag_source(&CString::new(include_str!("triangle.frag")).unwrap()).unwrap();
    let shader_program = render_gl::Program::from_shaders(&[vert_shader, frag_shader]).unwrap();



    //VR specific Stuff with OpenGL
    let mut framebuffer_name_right_eye: gl::types::GLuint = 0;
    let mut rendered_texture_right_eye: gl::types::GLuint = 0;

    let mut framebuffer_name_left_eye: gl::types::GLuint = 0;
    let mut rendered_texture_left_eye: gl::types::GLuint = 0;
    unsafe {
        //Right Eye
        gl::GenFramebuffers(1, &mut framebuffer_name_right_eye);
        gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer_name_right_eye);
        gl::GenTextures(1, &mut rendered_texture_right_eye);

        gl::BindTexture(gl::TEXTURE_2D, rendered_texture_right_eye);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, system.recommended_render_target_size().0 as i32, system.recommended_render_target_size().1 as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, 0 as *const std::ffi::c_void);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);

        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, rendered_texture_right_eye, 0);

        let draw_buffers: [gl::types::GLenum; 1] = [gl::COLOR_ATTACHMENT0];
        gl::DrawBuffers(1, draw_buffers.as_ptr());

        //Left Eye
        gl::GenFramebuffers(1, &mut framebuffer_name_left_eye);
        gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer_name_left_eye);
        gl::GenTextures(1, &mut rendered_texture_left_eye);

        gl::BindTexture(gl::TEXTURE_2D, rendered_texture_left_eye);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, system.recommended_render_target_size().0 as i32, system.recommended_render_target_size().1 as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, 0 as *const std::ffi::c_void);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);

        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, rendered_texture_left_eye, 0);

        let draw_buffers: [gl::types::GLenum; 1] = [gl::COLOR_ATTACHMENT0];
        gl::DrawBuffers(1, draw_buffers.as_ptr());
    }
    let tex_right_eye: openvr::compositor::Texture = openvr::compositor::Texture {
        handle: openvr::compositor::texture::Handle::OpenGLTexture(rendered_texture_right_eye as usize),
        color_space: openvr::compositor::texture::ColorSpace::Auto
    };
    let tex_left_eye: openvr::compositor::Texture = openvr::compositor::Texture {
        handle: openvr::compositor::texture::Handle::OpenGLTexture(rendered_texture_left_eye as usize),
        color_space: openvr::compositor::texture::ColorSpace::Auto
    };

    let mut empty_vao: gl::types::GLuint = 0;
    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer_name_right_eye);
        gl::Viewport(0, 0, system.recommended_render_target_size().0 as i32, system.recommended_render_target_size().1 as i32);
        gl::ClearColor(0.3, 0.3, 0.5, 1.0);

        gl::GenVertexArrays(1, &mut empty_vao);
    }

    let last_time = Instant::now();

    //-----------------------------------------------------------------


    // main loop
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        match comp.wait_get_poses() {
            Ok(sys) => sys,
            Err(err) => {
                println!("Couldn't get poses: {}", err);
                return;
            }
        };

        let elapsed = last_time.elapsed();

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        let pos = system.device_to_absolute_tracking_pose(openvr::TrackingUniverseOrigin::RawAndUncalibrated, 0.0);

        loop {
            match system.poll_next_event_with_pose(openvr::TrackingUniverseOrigin::RawAndUncalibrated) {
                None => break,
                Some(x) => x,
            };
        }


        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer_name_left_eye);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        shader_program.set_used();
        unsafe {
            gl::Uniform2f(2, system.recommended_render_target_size().0 as f32, system.recommended_render_target_size().1 as f32); //Screen Resolution
            gl::Uniform3f(3, 0.0, 0.0, 0.0); //Camera Position
            gl::Uniform1f(4, elapsed.as_secs() as f32 + (elapsed.subsec_millis() as f32 / 1_000f32)); //Time

            let eye_pos: f32 = (system.eye_to_head_transform(openvr::Eye::Left))[0][3];
            gl::Uniform1f(5, eye_pos);

            gl::BindVertexArray(empty_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer_name_right_eye);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        shader_program.set_used();
        unsafe {
            gl::Uniform2f(2, system.recommended_render_target_size().0 as f32, system.recommended_render_target_size().1 as f32); //Screen Resolution
            gl::Uniform3f(3, 0.0, 0.0, 0.0); //Camera Position
            gl::Uniform1f(4, elapsed.as_secs() as f32 + (elapsed.subsec_millis() as f32 / 1_000f32)); //Time

            let eye_pos: f32 = (system.eye_to_head_transform(openvr::Eye::Right))[0][3];
            gl::Uniform1f(5, eye_pos);

            gl::BindVertexArray(empty_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        let abs_hmd = *pos[0].device_to_absolute_tracking();
        unsafe {
            match comp.submit(openvr::Eye::Left, &tex_left_eye, None, Some(abs_hmd)) {
                Ok(sys) => sys,
                Err(err) => {
                    println!("Drawing Left Eye Failed: {}", err);
                    return;
                }
            }
            match comp.submit(openvr::Eye::Right, &tex_right_eye, None, Some(abs_hmd)) {
                Ok(sys) => sys,
                Err(err) => {
                    println!("Drawing Right Eye Failed: {}", err);
                    return;
                }
            }
        }
        window.gl_swap_window();
    }
}

fn f_4x3to4x4(mat: [[f32; 4]; 3]) -> [[f32; 4]; 4] {
    let empty_row = [0f32, 0f32, 0f32, 1f32];
    return [mat[0], mat[1], mat[2], empty_row];
}