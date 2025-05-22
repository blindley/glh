#[cfg(feature = "texture-loading")]
use std::path::Path;

use gl::types::*;
type Error = Box<dyn std::error::Error>;

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

pub fn create_texture_2d_grayscale_alpha(
    size: [i32; 2],
    data: &[u8],
) -> Result<GLuint, Error> {
    if data.len() != (size[0] * size[1] * 2) as usize {
        return Err(format!("create_texture_2d_grayscale_alpha: Data length does not match size: expected {}, got {}", size[0] * size[1] * 2, data.len()).into());
    }

    use detail::TextureFormat::GrayscaleAlpha;
    detail::create_texture_2d(size, data, GrayscaleAlpha)
        .map_err(|e| format!("create_texture_2d_grayscale_alpha: {}", e).into())
}

#[cfg(feature = "texture-loading")]
pub fn load_texture_2d<P: AsRef<Path>>(path: P) -> Result<GLuint, Error> {
    use stb_image::image::LoadResult::*;
    match stb_image::image::load(path) {
        ImageU8(img) => {
            let size = [img.width as i32, img.height as i32];
            match img.depth {
                1 => create_texture_2d_grayscale(size, &img.data),
                2 => create_texture_2d_grayscale_alpha(size, &img.data),
                3 => create_texture_2d_rgb(size, &img.data),
                4 => create_texture_2d_rgba(size, &img.data),
                _ => Err(format!("Unsupported image depth: {}", img.depth).into()),
            }
        }
        ImageF32(_) => {
            // Handle floating point images if needed
            Err("Floating point images are not currently supported".into())
        }
        Error(err) => {
            // Handle error
            Err(format!("Failed to load image: {}", err).into())
        }
    }
}

mod detail {
    use gl::types::*;
    use super::Error;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TextureFormat {
        RGB,
        RGBA,
        Grayscale,
        GrayscaleAlpha,
    }

    pub fn create_texture_2d(
        size: [i32; 2],
        data: &[u8],
        format: TextureFormat,
    ) -> Result<GLuint, Error> {
        let (internal_format, gl_format, pixel_size) = match format {
            TextureFormat::RGB => (gl::RGB8, gl::RGB, 3),
            TextureFormat::RGBA => (gl::RGBA8, gl::RGBA, 4),
            TextureFormat::Grayscale => (gl::R8, gl::RED, 1),
            TextureFormat::GrayscaleAlpha => (gl::RG8, gl::RG, 2),
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

            match format {
                TextureFormat::Grayscale | TextureFormat::GrayscaleAlpha =>{
                    // gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_SWIZZLE_R, gl::RED as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_SWIZZLE_G, gl::RED as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_SWIZZLE_B, gl::RED as i32);
                },
                _ => {}
            }

        }

        Ok(texture)
    }
}
