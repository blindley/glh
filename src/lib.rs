
use gl;
use gl::types::*;

type Error = Box<dyn std::error::Error>;

/// Checks for OpenGL errors and returns an error message if any are found.
pub fn get_error() -> Option<String> {
    let error_code = unsafe { gl::GetError() };
    if error_code != gl::NO_ERROR {
        let error_message = match error_code {
            gl::INVALID_ENUM => "GL_INVALID_ENUM: An unacceptable value is specified for an enumerated argument.",
            gl::INVALID_VALUE => "GL_INVALID_VALUE: A numeric argument is out of range.",
            gl::INVALID_OPERATION => "GL_INVALID_OPERATION: The specified operation is not allowed in the current state.",
            gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW: A stack pushing operation would overflow the maximum stack size.",
            gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW: A stack popping operation would underflow the minimum stack size.",
            gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY: There is not enough memory left to execute the command.",
            gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION: The framebuffer object is not complete.",
            _ => "Unknown OpenGL error",
        };
        Some(format!("OpenGL Error ({}): {}", error_code, error_message))
    } else {
        None
    }
}

/// Creates an OpenGL buffer and fills it with the provided data.
/// 
/// usage: The usage hint for the buffer.  See (https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferData.xhtml) for more information.
pub fn create_buffer<T: Copy>(
    data: &[T],
    usage: GLenum,
) -> Result<GLuint, Error> {
    if data.is_empty() {
        return Err("create_buffer(...): Data array is empty".into());
    }

    const VALID_USAGES: [GLenum; 9] = [
        gl::STREAM_DRAW,
        gl::STREAM_READ,
        gl::STREAM_COPY,
        gl::STATIC_DRAW,
        gl::STATIC_READ,
        gl::STATIC_COPY,
        gl::DYNAMIC_DRAW,
        gl::DYNAMIC_READ,
        gl::DYNAMIC_COPY,
    ];

    if !VALID_USAGES.contains(&usage) {
        return Err(format!("create_buffer(...): Invalid usage for buffer: {}; Must be one of [gl::STREAM_DRAW, gl::STREAM_READ, gl::STREAM_COPY, gl::STATIC_DRAW, gl::STATIC_READ, gl::STATIC_COPY, gl::DYNAMIC_DRAW, gl::DYNAMIC_READ, gl::DYNAMIC_COPY]", usage).into());
    }

    let mut buffer = 0;
    unsafe {
        // Clear any previous error before the call
        while gl::GetError() != gl::NO_ERROR {}

        gl::CreateBuffers(1, &mut buffer);
        if let Some(err_msg) = get_error() {
            return Err(format!("Failed to create buffer: {}", err_msg).into());
        }

        if buffer == 0 {
            return Err("gl::CreateBuffers returned an invalid buffer ID (0)".into());
        }

        let size = (data.len() * std::mem::size_of::<T>()) as isize;
        let data_ptr = data.as_ptr() as *const std::ffi::c_void;

        // Clear any previous error before the call
        while gl::GetError() != gl::NO_ERROR {}

        gl::NamedBufferData(buffer, size, data_ptr, usage);
        if let Some(err_msg) = get_error() {
            // If NamedBufferData fails, you should delete the buffer to avoid a leak
            gl::DeleteBuffers(1, &buffer);
            return Err(format!("Failed to set buffer data: {}", err_msg).into());
        }
    }
    Ok(buffer)
}

fn shader_type_name(shader_type: GLenum) -> Result<&'static str, Error> {
    let name = match shader_type {
        gl::VERTEX_SHADER => "Vertex",
        gl::FRAGMENT_SHADER => "Fragment",
        gl::GEOMETRY_SHADER => "Geometry",
        gl::TESS_CONTROL_SHADER => "Tess Control",
        gl::TESS_EVALUATION_SHADER => "Tess Evaluation",
        gl::COMPUTE_SHADER => "Compute",
        _ => return Err(format!("Invalid shader type: {}", shader_type).into()),
    };

    Ok(name)
}

pub fn compile_shader(source: &str, shader_type: GLenum) -> Result<GLuint, Error> {
    let shader_type_name = shader_type_name(shader_type)?;

    let shader = unsafe { gl::CreateShader(shader_type) };
    if shader == 0 {
        return Err(format!("Failed to create {} shader", shader_type_name).into());
    }

    let c_str = std::ffi::CString::new(source).map_err(|_| "Failed to convert source to CString")?;
    unsafe {
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        if status != (gl::TRUE as GLint) {
            let mut length = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut length);
            let mut info_log = vec![0; length as usize];
            gl::GetShaderInfoLog(shader, length, std::ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            let info_log = String::from_utf8_lossy(&info_log);
            gl::DeleteShader(shader);

            return Err(format!("{} Shader compilation failed: {}", shader_type_name, info_log).into());
        }
    }
    Ok(shader)
}

pub fn create_program(shaders: &[GLuint]) -> Result<GLuint, Error> {
    let program = unsafe { gl::CreateProgram() };
    if program == 0 {
        return Err("Failed to create program".into());
    }

    for &shader in shaders {
        unsafe {
            gl::AttachShader(program, shader);
        }
    }

    unsafe {
        gl::LinkProgram(program);
        for &shader in shaders {
            gl::DetachShader(program, shader);
        }

        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
        if status != (gl::TRUE as GLint) {
            let mut length = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut length);
            let mut info_log = vec![0; length as usize];
            gl::GetProgramInfoLog(program, length, std::ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            let info_log = String::from_utf8_lossy(&info_log);
            gl::DeleteProgram(program);

            return Err(format!("Program linking failed: {}", info_log).into());
        }
    }

    Ok(program)
}

pub struct ProgramBuilder {
    shaders: std::collections::HashMap<GLenum, GLuint>,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self {
            shaders: std::collections::HashMap::new(),
        }
    }

    pub fn with_shader(mut self, shader_type: GLenum, source: &str) -> Result<Self, Error> {
        if self.shaders.contains_key(&shader_type) {
            let name = shader_type_name(shader_type)?;
            return Err(format!("{} Shader type already added", name).into());
        }

        let shader = compile_shader(source, shader_type)?;
        self.shaders.insert(shader_type, shader);
        Ok(self)
    }

    pub fn with_vertex_shader(mut self, source: &str) -> Result<Self, Error> {
        self = self.with_shader(gl::VERTEX_SHADER, source)?;
        Ok(self)
    }

    pub fn with_fragment_shader(mut self, source: &str) -> Result<Self, Error> {
        self = self.with_shader(gl::FRAGMENT_SHADER, source)?;
        Ok(self)
    }

    pub fn with_geometry_shader(mut self, source: &str) -> Result<Self, Error> {
        self = self.with_shader(gl::GEOMETRY_SHADER, source)?;
        Ok(self)
    }

    pub fn with_tess_control_shader(mut self, source: &str) -> Result<Self, Error> {
        self = self.with_shader(gl::TESS_CONTROL_SHADER, source)?;
        Ok(self)
    }

    pub fn with_tess_evaluation_shader(mut self, source: &str) -> Result<Self, Error> {
        self = self.with_shader(gl::TESS_EVALUATION_SHADER, source)?;
        Ok(self)
    }

    pub fn with_compute_shader(mut self, source: &str) -> Result<Self, Error> {
        self = self.with_shader(gl::COMPUTE_SHADER, source)?;
        Ok(self)
    }

    pub fn build(self) -> Result<GLuint, Error> {
        if self.shaders.is_empty() {
            return Err("No shaders added to the program".into());
        }

        let shader_ids: Vec<GLuint> = self.shaders.values().cloned().collect();
        create_program(&shader_ids)
    }
}

impl std::ops::Drop for ProgramBuilder {
    fn drop(&mut self) {
        for &shader in self.shaders.values() {
            unsafe {
                gl::DeleteShader(shader);
            }
        }
    }
}

/// Enables a series of interleaved vertex array attributes all of the same type and in the same buffer.
/// 
/// Warning: Global OpenGL bindings may be modified by this function.
pub fn enable_interleaved_vertex_array_attributes(
    vao: GLuint,
    buffer: GLuint,
    type_: GLenum,
    normalized: bool,
    start_index: i32,
    sizes: &[i32],
) -> Result<(), Error> {
    if sizes.is_empty() {
        return Err("enable_interleaved_vertex_attributes: Sizes array is empty".into());
    }

    let component_size = match type_ {
        gl::FLOAT => std::mem::size_of::<GLfloat>(),
        gl::DOUBLE => std::mem::size_of::<GLdouble>(),
        gl::BYTE => std::mem::size_of::<GLbyte>(),
        gl::UNSIGNED_BYTE => std::mem::size_of::<GLubyte>(),
        gl::SHORT => std::mem::size_of::<GLshort>(),
        gl::UNSIGNED_SHORT => std::mem::size_of::<GLushort>(),
        gl::INT => std::mem::size_of::<GLint>(),
        gl::UNSIGNED_INT => std::mem::size_of::<GLuint>(),
        _ => return Err(format!("enable_interleaved_vertex_attributes: Invalid type: {}", type_).into()),
    } as i32;

    let stride = sizes.iter().sum::<i32>() * component_size;

    unsafe {
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);

        let mut offset = 0;
        for (index_offset, &size) in sizes.iter().enumerate() {
            let index = start_index + index_offset as i32;
            gl::EnableVertexAttribArray(index as GLuint);
            gl::VertexAttribPointer(
                index as GLuint,
                size,
                type_,
                if normalized { gl::TRUE } else { gl::FALSE },
                stride,
                offset as *const std::ffi::c_void,
            );

            offset += size * component_size;
        }
    }

    Ok(())
}
