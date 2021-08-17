use aperture_common::VPos;
use aperture_mesh::{ImageFormat, Texture};

use image::hdr::HdrDecoder;

use std::fs::File;
use std::io::BufReader;

pub struct Cube {
    pub texture: Texture<f32>,
}

impl Cube {
    pub const NUM_VERTICES: usize = 36;

    pub const VERTICES: [VPos; Self::NUM_VERTICES] = [
        VPos { position: [-1.0, -1.0, -1.0] },
        VPos { position: [-1.0, -1.0,  1.0] },
        VPos { position: [-1.0,  1.0,  1.0] },
        VPos { position: [ 1.0,  1.0, -1.0] },
        VPos { position: [-1.0, -1.0, -1.0] },
        VPos { position: [-1.0,  1.0, -1.0] },

        VPos { position: [ 1.0, -1.0,  1.0] },
        VPos { position: [-1.0, -1.0, -1.0] },
        VPos { position: [ 1.0, -1.0, -1.0] },
        VPos { position: [ 1.0,  1.0, -1.0] },
        VPos { position: [ 1.0, -1.0, -1.0] },
        VPos { position: [-1.0, -1.0, -1.0] },

        VPos { position: [-1.0, -1.0, -1.0] },
        VPos { position: [-1.0,  1.0,  1.0] },
        VPos { position: [-1.0,  1.0, -1.0] },
        VPos { position: [ 1.0, -1.0,  1.0] },
        VPos { position: [-1.0, -1.0,  1.0] },
        VPos { position: [-1.0, -1.0, -1.0] },
        
        VPos { position: [-1.0,  1.0,  1.0] },
        VPos { position: [-1.0, -1.0,  1.0] },
        VPos { position: [ 1.0, -1.0,  1.0] },
        VPos { position: [ 1.0,  1.0,  1.0] },
        VPos { position: [ 1.0, -1.0, -1.0] },
        VPos { position: [ 1.0,  1.0, -1.0] },
        
        VPos { position: [ 1.0, -1.0, -1.0] },
        VPos { position: [ 1.0,  1.0,  1.0] },
        VPos { position: [ 1.0, -1.0,  1.0] },
        VPos { position: [ 1.0,  1.0,  1.0] },
        VPos { position: [ 1.0,  1.0, -1.0] },
        VPos { position: [-1.0,  1.0, -1.0] },
        
        VPos { position: [ 1.0,  1.0,  1.0] },
        VPos { position: [-1.0,  1.0, -1.0] },
        VPos { position: [-1.0,  1.0,  1.0] },
        VPos { position: [ 1.0,  1.0,  1.0] },
        VPos { position: [-1.0,  1.0,  1.0] },
        VPos { position: [ 1.0, -1.0,  1.0] },
    ];
    
    pub fn textured() -> Self {
        let hdr = HdrDecoder::new(
            BufReader::new(File::open("data/images/environment.hdr").unwrap())
        )
        .unwrap();

        let (width, height) = (hdr.metadata().width, hdr.metadata().height);
        let hdr_data = hdr.read_image_hdr().unwrap();

        let mut pixel_data = vec![];
        for pixel in hdr_data {
            pixel_data.push(pixel.0[0]);
            pixel_data.push(pixel.0[1]);
            pixel_data.push(pixel.0[2]);
            pixel_data.push(1.0);
        }

        let mut texture = Texture::default();
        texture.name = "env_cubemap".to_string();
        texture.format = ImageFormat::R32G32B32A32;
        texture.pixels = pixel_data;
        texture.width = width;
        texture.height = height;

        Self { texture }
    }
}
