
use gl::types::*;
type Error = Box<dyn std::error::Error>;

pub fn compile_shader(source: &str, shader_type: GLenum) -> Result<GLuint, Error> {
    let shader_type_name = detail::shader_type_name(shader_type)?;

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
            let name = detail::shader_type_name(shader_type)?;
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

mod detail {
    use gl::types::*;
    use super::Error;

    pub fn shader_type_name(shader_type: GLenum) -> Result<&'static str, Error> {
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
}
