
use gl::types::*;
type Error = Box<dyn std::error::Error>;

pub fn create_texture_2d_grayscale(
    size: [i32; 2],
    data: &[u8],
) -> Result<GLuint, Error> {
    if data.len() != (size[0] * size[1]) as usize {
        return Err(format!("create_texture_2d_grayscale: Data length does not match size: expected {}, got {}", size[0] * size[1], data.len()).into());
    }

    use detail::TextureFormat::Grayscale;
    detail::create_texture_2d(size, data, Grayscale)
        .map_err(|e| format!("create_texture_2d_grayscale: {}", e).into())
}

pub fn create_texture_2d_rgb(
    size: [i32; 2],
    data: &[u8],
) -> Result<GLuint, Error> {
    if data.len() != (size[0] * size[1] * 3) as usize {
        return Err(format!("create_texture_2d_rgb: Data length does not match size: expected {}, got {}", size[0] * size[1] * 3, data.len()).into());
    }

    use detail::TextureFormat::RGB;
    detail::create_texture_2d(size, data, RGB)
        .map_err(|e| format!("create_texture_2d_rgb: {}", e).into())
}

pub fn create_texture_2d_rgba(
    size: [i32; 2],
    data: &[u8],
) -> Result<GLuint, Error> {
    if data.len() != (size[0] * size[1] * 4) as usize {
        return Err(format!("create_texture_2d_rgba: Data length does not match size: expected {}, got {}", size[0] * size[1] * 4, data.len()).into());
    }

    use detail::TextureFormat::RGBA;
    detail::create_texture_2d(size, data, RGBA)
        .map_err(|e| format!("create_texture_2d_rgba: {}", e).into())
}

mod detail {
    use gl::types::*;
    use super::Error;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TextureFormat {
        Grayscale,
        RGB,
        RGBA,
    }

    pub fn create_texture_2d(
        size: [i32; 2],
        data: &[u8],
        format: TextureFormat,
    ) -> Result<GLuint, Error> {
        let (internal_format, gl_format, pixel_size) = match format {
            TextureFormat::Grayscale => (gl::R8, gl::RED, 1),
            TextureFormat::RGB => (gl::RGB8, gl::RGB, 3),
            TextureFormat::RGBA => (gl::RGBA8, gl::RGBA, 4),
        };

        if data.len() != (size[0] * size[1] * pixel_size) as usize {
            return Err(format!("create_texture_2d: Data length does not match size: expected {}, got {}", size[0] * size[1] * pixel_size, data.len()).into());
        }

        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            if texture == 0 {
                return Err("Failed to generate texture".into());
            }

            gl::BindTexture(gl::TEXTURE_2D, texture);
            if gl::GetError() != gl::NO_ERROR {
                return Err("Failed to bind texture".into());
            }

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format as i32,
                size[0],
                size[1],
                0,
                gl_format,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const std::ffi::c_void,
            );

            // Set common texture parameters for completeness
            gl::TextureParameteri(texture, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TextureParameteri(texture, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TextureParameteri(texture, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TextureParameteri(texture, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        }

        Ok(texture)
    }
}
