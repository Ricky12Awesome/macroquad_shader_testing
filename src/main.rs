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

unsafe fn compile_compute_shader(source: &[u8]) -> Result<GLuint, ()> {
  let mut source_gl = source.to_vec();

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

    return Err(());
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

    return Err(());
  }

  Ok(program)
}

unsafe fn run_compute_shader<T>(program: GLuint, default: T, layout: (usize, usize, usize)) -> T
  where T: Sized + Copy
{
  glUseProgram(program);
  let mut ssbo = 0;
  let mut ssbo_data = default;
  glGenBuffers(1, &mut ssbo);
  glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 0, ssbo);

  glBufferData(
    GL_SHADER_STORAGE_BUFFER,
    size_of::<T>() as _,
    &ssbo_data as *const T as _,
    GL_DYNAMIC_DRAW,
  );

  glDispatchCompute(layout.0 as _, layout.1 as _, layout.2 as _);
  glMemoryBarrier(GL_SHADER_STORAGE_BARRIER_BIT);
  glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 0, 0);

  glBindBuffer(GL_SHADER_STORAGE_BUFFER, ssbo);

  ssbo_data = *(glMapBuffer(GL_SHADER_STORAGE_BUFFER, GL_READ_ONLY) as *const T);

  glUnmapBuffer(GL_SHADER_STORAGE_BUFFER);

  ssbo_data
}

// Using the macro for the main function makes code-completion not work for that function,
// so that's why this function exists
async unsafe fn _main() {
  const NOTO_SANS: &[u8] = include_bytes!("/usr/share/fonts/noto/NotoSans-Regular.ttf");

  let program = compile_compute_shader(include_bytes!("../resources/shader.comp")).unwrap();
  let data = run_compute_shader(program, [0; 32 * 32], (32, 32, 1));

  println!("{data:?} {}", data.iter().sum::<u32>());
  println!("{}", data.iter().len());

  let mut fonts = macroquad_text::Fonts::default();

  fonts.load_font_from_bytes("Noto Sans Regular", NOTO_SANS).unwrap();

  loop {
    next_frame().await;
  }
}

