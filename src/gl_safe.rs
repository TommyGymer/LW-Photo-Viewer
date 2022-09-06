use glitz::{GlFns, GL_COLOR_BUFFER_BIT, gl_types::*, gl_constants::*};

pub fn clear_colour(gl: &GlFns, r: f32, g: f32, b: f32, a: f32) {
    unsafe{ gl.ClearColor(r,g,b,a) }
}

pub struct VertexArray(pub GLuint);
impl VertexArray {
    pub fn new(gl: &GlFns) -> Option<Self> {
        let mut vao = 0;
        unsafe { gl.GenVertexArrays(1, &mut vao) };
        if vao != 0 {
            Some(Self(vao))
        }else{
            None
        }
    }

    pub fn bind(&self, gl: &GlFns) {
        unsafe { gl.BindVertexArray(self.0) }
    }
}

pub fn clear_vertex_binding(gl: &GlFns) {
    unsafe { gl.BindVertexArray(0) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    Array = GL_ARRAY_BUFFER as isize,
    ElementArray = GL_ELEMENT_ARRAY_BUFFER as isize,
}

pub struct Buffer(pub GLuint);
impl Buffer {
    pub fn new(gl: &GlFns) -> Option<Self> {
        let mut vbo = 0;
        unsafe { gl.GenBuffers(1, &mut vbo) };
        if vbo != 0 {
            Some(Self(vbo))
        }else{
            None
        }
    }

    pub fn bind(&self, gl: &GlFns, ty: BufferType) {
        unsafe { gl.BindBuffer(ty as GLenum, self.0) }
    }
}

pub fn clear_buffer_binding(gl: &GlFns, ty: BufferType) {
    unsafe { gl.BindBuffer(ty as GLenum, 0) }
}

pub fn buffer_data(gl: &GlFns, ty: BufferType, data: &[u8], usage: GLenum) {
    unsafe {
        gl.BufferData(
            ty as GLenum,
            data.len().try_into().unwrap(),
            data.as_ptr().cast(),
            usage,
        )
    }
}

pub enum ShaderType {
    Vertex = GL_VERTEX_SHADER as isize,
    Fragment = GL_FRAGMENT_SHADER as isize,
}

pub struct Shader(pub GLuint);
impl Shader {
    pub fn new(gl: &GlFns, ty: ShaderType) -> Option<Self> {
        let shader = unsafe { gl.CreateShader(ty as GLenum) };
        if shader != 0 {
            Some(Self(shader))
        }else{
            None
        }
    }

    pub fn set_source(&self, gl: &GlFns, src: &str) {
        unsafe {
            gl.ShaderSource(
                self.0,
                1,
                src.as_bytes().as_mut_ptr().cast(),
                &(src.len().try_into().unwrap()),
            );
        }
    }

    pub fn compile(&self, gl: &GlFns) {
        unsafe { gl.CompileShader(self.0) }
    }

    pub fn compile_success(&self, gl: &GlFns) -> bool {
        let mut compiled = 0;
        unsafe { gl.GetShaderiv(self.0, GL_COMPILE_STATUS, &mut compiled) };
        compiled == GL_TRUE.try_into().unwrap()
    }

    pub fn info_log(&self, gl: &GlFns) -> String {
        let mut needed_len = 0;
        unsafe { gl.GetShaderiv(self.0, GL_INFO_LOG_LENGTH, &mut needed_len) };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            gl.GetShaderInfoLog(
                self.0,
                v.capacity().try_into().unwrap(),
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    pub fn delete(self, gl: &GlFns) {
        unsafe { gl.DeleteShader(self.0) };
    }

    pub fn from_source(gl: &GlFns, ty: ShaderType, source: &str) -> Result<Self, String> {
        println!("Creating shader from source");
        let id = Self::new(&gl, ty)
            .ok_or_else(|| "Couldn't allocate new shader".to_string())?;
        println!("Shader allocated");
        id.set_source(&gl, source);
        println!("Source set");
        id.compile(&gl);
        println!("hmm");
        if id.compile_success(&gl) {
            Ok(id)
        } else {
            let out = id.info_log(&gl);
            id.delete(&gl);
            Err(out)
        }
    }
}

pub struct ShaderProgram(pub GLuint);
impl ShaderProgram {
    pub fn new(gl: &GlFns) -> Option<Self> {
        let prog = unsafe { gl.CreateProgram() };
        if prog != 0 {
            Some(Self(prog))
        } else {
            None
        }
    }

    pub fn attach_shader(&self, gl: &GlFns, shader: &Shader) {
        unsafe { gl.AttachShader(self.0, shader.0 ) }
    }

    pub fn link_program(&self, gl: &GlFns) {
        unsafe { gl.LinkProgram(self.0) }
    }

    pub fn link_success(&self, gl: &GlFns) -> bool {
        let mut success = 0;
        unsafe { gl.GetProgramiv(self.0, GL_LINK_STATUS, &mut success) };
        success == GL_TRUE.try_into().unwrap()
    }

    pub fn info_log(&self, gl: &GlFns) -> String {
        let mut needed_len = 0;
        unsafe { gl.GetProgramiv(self.0, GL_INFO_LOG_LENGTH, &mut needed_len) };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            gl.GetProgramInfoLog(
                self.0,
                v.capacity().try_into().unwrap(),
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    pub fn use_program(&self, gl: &GlFns) {
        unsafe { gl.UseProgram(self.0) }
    }

    pub fn delete(self, gl: &GlFns){
        unsafe { gl.DeleteProgram(self.0) }
    }

    pub fn from_vert_frag(gl: &GlFns, vert: &str, frag: &str) -> Result<Self, String> {
        let p =
            Self::new(&gl).ok_or_else(|| "Couldn't allocate a program".to_string())?;
        println!("Created program");
        let v = Shader::from_source(&gl, ShaderType::Vertex, vert)
            .map_err(|e| format!("Vertex Compile Error: {}", e))?;
        println!("Created vertex shader");
        let f = Shader::from_source(&gl, ShaderType::Fragment, frag)
            .map_err(|e| format!("Fragment Compile Error: {}", e))?;
        println!("Created fragment shader");
        println!("Loaded shaders");
        p.attach_shader(&gl, &v);
        p.attach_shader(&gl, &f);
        p.link_program(&gl);
        v.delete(&gl);
        f.delete(&gl);
        if p.link_success(&gl) {
            Ok(p)
        } else {
            let out = format!("Program Link Error: {}", p.info_log(&gl));
            p.delete(&gl);
            Err(out)
        }
    }
}