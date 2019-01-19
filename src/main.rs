extern crate gl;
extern crate sdl2;
extern crate openvr;
//extern crate cgmath;

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

    let mut r_proj_raw: openvr::system::RawProjection = system.projection_raw(openvr::Eye::Right);
    let r_proj_prop = [r_proj_raw.left, r_proj_raw.right, r_proj_raw.bottom, r_proj_raw.top];
    r_proj_raw = system.projection_raw(openvr::Eye::Left);
    let l_proj_prop = [r_proj_raw.left, r_proj_raw.right, r_proj_raw.bottom, r_proj_raw.top];

    println!("Right Eye Prop: {:?}", r_proj_prop);
    println!("Light Eye Prop: {:?}", l_proj_prop);

    println!("Right Eye Matrix: {:?}", system.eye_to_head_transform(openvr::Eye::Right)[0][3]);
    println!("Left Eye Matrix: {:?}", system.eye_to_head_transform(openvr::Eye::Left)[0][3]);
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


        let pos = system.device_to_absolute_tracking_pose(openvr::TrackingUniverseOrigin::Standing, 0.0);
        let hmd_wr_pos = *pos[0].device_to_absolute_tracking();
        let hmd_pos = get_tracking_position(hmd_wr_pos);
        let hmd_rot = get_tracking_rotation(hmd_wr_pos);

        loop {
            match system.poll_next_event_with_pose(openvr::TrackingUniverseOrigin::Standing) {
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

            gl::Uniform4f(6, l_proj_prop[0], l_proj_prop[1], l_proj_prop[2], l_proj_prop[3]);
            //gl::UniformMatrix4fv(7, 1, gl::FALSE, std::mem::transmute(&hmd_pos));

            gl::Uniform3f(8, hmd_pos[0], hmd_pos[1], hmd_pos[2]);
            gl::UniformMatrix4fv(9, 1, gl::FALSE, std::mem::transmute(&hmd_rot));

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

            gl::Uniform4f(6, r_proj_prop[0], r_proj_prop[1], r_proj_prop[2], r_proj_prop[3]);
            //gl::UniformMatrix4fv(7, 1, gl::FALSE, std::mem::transmute(&hmd_pos));

            gl::Uniform3f(8, hmd_pos[0], hmd_pos[1], hmd_pos[2]);
            gl::UniformMatrix4fv(9, 1, gl::FALSE, std::mem::transmute(&hmd_rot));

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

fn get_tracking_rotation(mat: [[f32; 4]; 3]) -> [[f32; 4]; 4] {
    /*return [
        (1f32 + mat[0][0] + mat[1][1] + mat[2][2]).max(0f32).sqrt() / 2f32,

        copysign((1f32 + mat[0][0] - mat[1][1] - mat[2][2]).max(0f32).sqrt() / 2f32, mat[2][1] - mat[1][2]),
        copysign((1f32 - mat[0][0] + mat[1][1] - mat[2][2]).max(0f32).sqrt() / 2f32, mat[0][2] - mat[2][0]),
        copysign((1f32 - mat[0][0] - mat[1][1] + mat[2][2]).max(0f32).sqrt() / 2f32, mat[1][0] - mat[0][1])
    ];*/

    let sx = (mat[0][0].powi(2) + mat[1][0].powi(2) + mat[2][0].powi(2)).sqrt();
    let sy = (mat[0][1].powi(2) + mat[1][1].powi(2) + mat[2][1].powi(2)).sqrt();
    let sz = (mat[0][2].powi(2) + mat[1][2].powi(2) + mat[2][2].powi(2)).sqrt();

    return [
        [mat[0][0] / sx, mat[1][0] / sy, mat[2][0], 0f32],
        [mat[0][1] / sx, mat[1][1] / sy, mat[2][1], 0f32],
        [mat[0][2] / sx, mat[1][2] / sy, mat[2][2], 0f32],
        [0f32,           0f32,           0f32,      1f32]
    ];
}

fn get_tracking_position(mat: [[f32; 4]; 3]) -> [f32; 3] {
    return [
        mat[0][3],
        mat[1][3],
        mat[2][3]
    ];
}

/*fn get_tracking_yrp(mat: [[f32; 4]; 3]) -> [f32; 3] {

}*/

fn copysign(to: f32, from: f32) -> f32 {
    if (from < 0f32 && to < 0f32) || (from >= 0f32 && to >= 0f32) {
        return to;
    } else {
        return -to;
    }
}

/*fn f_4x3to4x4(mat: [[f32; 4]; 3]) -> [[f32; 4]; 4] {
    let empty_row = [0f32, 0f32, 0f32, 1f32];
    return [mat[0], mat[1], mat[2], empty_row];
}*/