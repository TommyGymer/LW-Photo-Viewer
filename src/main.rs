//make use of https://crates.io/crates/image to decode main image formats
//make use of https://github.com/pedrocr/rawloader/ for decoding of RAW images
//make use of https://docs.rs/druid/0.7.0/druid/widget/struct.Image.html the druid image widget for displaying images

#![allow(unused)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use beryllium::{
        event::Event,
        gl_window::{GlAttr, GlContextFlags, GlProfile},
        init::{InitFlags, Sdl},
        window::WindowFlags,
        SdlResult,
    };
use glitz::{println_gl_debug_callback, GlFns, GL_COLOR_BUFFER_BIT, gl_types::*, gl_constants::*};
//use ogl33::*;
use core::{
    convert::{TryInto},
    mem::{size_of, size_of_val},
  };
use bytemuck;
use std::time::{Duration, Instant};
use image::io::Reader as ImageReader;
use zstring::zstr;

mod gl_safe;

//pos, uv
type Vertex = [f32; 3 + 2];
type TriIndexes = [u32; 3];

//triangle data
// const TRIANGLE: [Vertex; 3] =
//     [
//         [-0.5, -0.5, 0.0],
//         [0.5, -0.5, 0.0],
//         [0.0, 0.5, 0.0]
//     ];

const RECTANGLE: [Vertex; 4] = 
    [
        [1.0, 1.0, 0.0, 1.0, 0.0],
        [1.0, -1.0, 0.0, 1.0, 1.0],
        [-1.0, -1.0, 0.0, 0.0, 1.0],
        [-1.0, 1.0, 0.0, 0.0, 0.0]
    ];

const INDICES: [TriIndexes; 2] = [
    [0, 1, 3],
    [1, 2, 3]
];

//vertex shader program
const VERT_SHADER: &str = r#"#version 330 core
  layout (location = 0) in vec3 pos;
  layout (location = 1) in vec2 uv;

  out vec2 frag_uv;

  void main() {
    gl_Position = vec4(pos, 1.0);
    frag_uv = uv;
  }
"#;

const FRAG_SHADER: &str = r#"#version 330 core
  uniform sampler2D tx;

  in vec2 frag_uv;

  out vec4 final_colour;

  void main() {
    final_colour = texture(tx, frag_uv);
  }
"#;

fn main() {
    let bitmap = {
        let data = ImageReader::open("C:\\Users\\Tom\\Pictures\\写真\\1116473.jpg").unwrap();
        let img = data.decode().unwrap();
        img.into_rgba8()
    };
    println!("Image loaded");

    let sdl = Sdl::init(InitFlags::EVERYTHING).expect("couldn't start SDL");
    sdl.gl_set_attribute(GlAttr::MajorVersion, 3).unwrap();
    sdl.gl_set_attribute(GlAttr::MinorVersion, 3).unwrap();
    sdl.gl_set_attribute(GlAttr::Profile, GlProfile::Core as _).unwrap();
    #[cfg(target_os = "macos")]
    {
        sdl
        .gl_set_attribute(SdlGlAttr::Flags, ContextFlag::ForwardCompatible)
        .unwrap();
    }

    println!("SDL loaded");

    let win = sdl
        .create_gl_window(
            zstr!("Hello Window"),
            Some((0, 0)),
            (800, 600),
            WindowFlags::RESIZABLE,
        )
        .expect("couldn't make a window and context");

    println!("Window created");

    let gl = unsafe { GlFns::from_loader(&|zs| win.get_proc_address(zs)).unwrap() };
    if win.is_extension_supported(zstr!("GL_KHR_debug")) {
        println!("Activating the debug callback...");
        unsafe { gl.DebugMessageCallback(Some(println_gl_debug_callback), 0 as *const _) };
    }

    println!("gl loaded");

    let vao = gl_safe::VertexArray::new(&gl).expect("Couldn't make a VAO");
    vao.bind(&gl);

    //vertex buffer
    let vbo = gl_safe::Buffer::new(&gl).expect("Couldn't make a VBO");
    vbo.bind(&gl, gl_safe::BufferType::Array);
    gl_safe::buffer_data(
        &gl,
        gl_safe::BufferType::Array,
        bytemuck::cast_slice(&RECTANGLE),
        GL_STATIC_DRAW,
    );

    let ebo = gl_safe::Buffer::new(&gl).expect("Couldn't make the element buffer");
    ebo.bind(&gl, gl_safe::BufferType::ElementArray);
    gl_safe::buffer_data(
        &gl,
        gl_safe::BufferType::ElementArray,
        bytemuck::cast_slice(&INDICES),
        GL_STATIC_DRAW,
    );

    println!("Buffer objects created");

    let mut texture = 0;
    unsafe {
        gl.GenTextures(1, &mut texture);
        gl.BindTexture(GL_TEXTURE_2D, texture);
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT as GLint);
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT as GLint);
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as GLint);
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as GLint);
        gl.TexImage2D(
            GL_TEXTURE_2D,
            0,
            GL_RGBA as GLint,
            bitmap.width().try_into().unwrap(),
            bitmap.height().try_into().unwrap(),
            0,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            bitmap.as_ptr().cast(),
        );
        gl.GenerateMipmap(GL_TEXTURE_2D);
    }

    println!("Texture loaded");

    unsafe {
        //define the data as vertex data
        gl.VertexAttribPointer(
            0,
            3,
            GL_FLOAT,
            GL_FALSE.try_into().unwrap(),
            size_of::<Vertex>().try_into().unwrap(),
            0 as *const _,
        );
        gl.EnableVertexAttribArray(0);

        gl.VertexAttribPointer(
            1,
            2,
            GL_FLOAT,
            GL_FALSE.try_into().unwrap(),
            size_of::<Vertex>().try_into().unwrap(),
            size_of::<[f32; 3]>() as *const _,
        );
        gl.EnableVertexAttribArray(1);
    }

    println!("Attribs set");

    //shader program
    let shader_program = gl_safe::ShaderProgram::from_vert_frag(&gl, VERT_SHADER, FRAG_SHADER).unwrap();
    println!("aaa");
    shader_program.use_program(&gl);

    println!("Shader program loaded");

    gl_safe::clear_colour(&gl, 0.2, 0.3, 0.3, 1.0);

    win.set_swap_interval(1);

    println!("Loop has started");

    'main_loop: loop {
        // handle events this frame
        while let Some(event) = sdl.poll_event() {
            match event {
            Event::Quit => break 'main_loop,
            _ => (),
            }
        }

        unsafe {
            gl.Clear(GL_COLOR_BUFFER_BIT);
            gl.DrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0 as *const _);
            //glDrawArrays(GL_TRIANGLES, 0, 3);
        }

        win.swap_backbuffer();
    }
}
