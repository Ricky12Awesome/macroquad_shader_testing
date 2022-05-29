use std::ffi::CStr;
use std::mem::size_of;

use macroquad::miniquad::gl::*;
use macroquad::prelude::*;

fn window_conf() -> Conf {
  Conf {
    window_width: 1920,
    window_height: 1080,
    window_resizable: true,
    window_title: String::from("Rust Shader Testing"),
    ..Conf::default()
  }
}

#[macroquad::main(window_conf)]
async fn main() {
  unsafe {
    _main().await;
  }
}

unsafe fn run_compute_shader_glsl() {
  const SOURCE_BYTES: &[u8] = include_bytes!("../resources/shader.comp");

  let mut source_gl = SOURCE_BYTES.to_vec();

  source_gl.push(0);  // null-termination

  let source_gl_ptr = source_gl.as_ptr() as *const _;

  let shader = glCreateShader(GL_COMPUTE_SHADER);
  glShaderSource(shader, 1, [source_gl_ptr].as_ptr(), std::ptr::null());
  glCompileShader(shader);

  let mut result = 0;
  glGetShaderiv(shader, GL_COMPILE_STATUS, &mut result);

  if result == 0 {
    println!("Error compiling shader");

    let mut log_buf = [0; 10240];
    let mut length = 0;

    glGetShaderInfoLog(
      shader,
      10240 - 1,
      &mut length,
      log_buf.as_mut_ptr(),
    );

    println!("ERR: {:?}", CStr::from_ptr(log_buf.as_ptr()));

    return;
  }

  let program = glCreateProgram();
  glAttachShader(program, shader);
  glLinkProgram(program);

  let mut result = 0;
  glGetProgramiv(program, GL_LINK_STATUS, &mut result);

  if result == 0 {
    println!("Error linking program");

    let mut log_buf = [0; 10240];
    let mut length = 0;

    glGetProgramInfoLog(
      program,
      10240 - 1,
      &mut length,
      log_buf.as_mut_ptr(),
    );

    println!("ERR: {:?}", CStr::from_ptr(log_buf.as_ptr()));

    return;
  }

  glUseProgram(program);
  let mut ssbo = 0;
  glGenBuffers(1, &mut ssbo);
  glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 0, ssbo);
  glBufferData(GL_SHADER_STORAGE_BUFFER, size_of::<GLuint>() as _, 0u32 as _, GL_DYNAMIC_DRAW);

  glDispatchCompute(1, 1, 1);
  glMemoryBarrier(GL_SHADER_STORAGE_BARRIER_BIT);
  glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 0, 0);

  glBindBuffer(GL_SHADER_STORAGE_BUFFER, ssbo);

  let ptr = glMapBuffer(GL_SHADER_STORAGE_BUFFER, GL_READ_ONLY) as *const GLuint;

  println!("{}", ptr.read());

  glUnmapBuffer(GL_SHADER_STORAGE_BUFFER);
}

// Using the macro for the main function makes code-completion not work for that function,
// so that's why this function exists
async unsafe fn _main() {
  const NOTO_SANS: &[u8] = include_bytes!("/usr/share/fonts/noto/NotoSans-Regular.ttf");

  run_compute_shader_glsl();

  let mut fonts = macroquad_text::Fonts::default();

  fonts.load_font_from_bytes("Noto Sans Regular", NOTO_SANS).unwrap();

  loop {
    next_frame().await;
  }
}

