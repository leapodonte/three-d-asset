use crate::{io::RawAssets, texture::*, Result};
use image::{io::Reader, *};
use std::io::Cursor;
use std::path::Path;

pub fn deserialize_img(bytes: &[u8]) -> Result<Texture2D> {
    let reader = Reader::new(Cursor::new(bytes))
        .with_guessed_format()
        .expect("Cursor io never fails");
    #[cfg(feature = "hdr")]
    if reader.format() == Some(image::ImageFormat::Hdr) {
        use image::codecs::hdr::*;
        let decoder = HdrDecoder::new(&*bytes)?;
        let metadata = decoder.metadata();
        let img = decoder.read_image_native()?;
        return Ok(Texture2D {
            data: TextureData::RgbF32(
                img.iter()
                    .map(|rgbe| {
                        let Rgb(values) = rgbe.to_hdr();
                        [values[0], values[1], values[2]]
                    })
                    .collect::<Vec<_>>(),
            ),
            width: metadata.width,
            height: metadata.height,
            ..Default::default()
        });
    }
    let img: DynamicImage = reader.decode()?;
    let width = img.width();
    let height = img.height();
    let data = match img {
        DynamicImage::ImageLuma8(_) => TextureData::RU8(img.into_bytes()),
        DynamicImage::ImageLumaA8(_) => {
            let bytes = img.as_bytes();
            let mut data = Vec::new();
            for i in 0..bytes.len() / 2 {
                data.push([bytes[i * 2], bytes[i * 2 + 1]]);
            }
            TextureData::RgU8(data)
        }
        DynamicImage::ImageRgb8(_) => {
            let bytes = img.as_bytes();
            let mut data = Vec::new();
            for i in 0..bytes.len() / 3 {
                data.push([bytes[i * 3], bytes[i * 3 + 1], bytes[i * 3 + 2]]);
            }
            TextureData::RgbU8(data)
        }
        DynamicImage::ImageRgba8(_) => {
            let bytes = img.as_bytes();
            let mut data = Vec::new();
            for i in 0..bytes.len() / 4 {
                data.push([
                    bytes[i * 4],
                    bytes[i * 4 + 1],
                    bytes[i * 4 + 2],
                    bytes[i * 4 + 3],
                ]);
            }
            TextureData::RgbaU8(data)
        }
        _ => unimplemented!(),
    };
    Ok(Texture2D {
        data,
        width,
        height,
        ..Default::default()
    })
}

pub fn serialize_img(tex: &Texture2D, path: impl AsRef<Path>) -> Result<RawAssets> {
    let img = match &tex.data {
        TextureData::RgbaU8(data) => DynamicImage::ImageRgba8(
            ImageBuffer::from_raw(
                tex.width,
                tex.height,
                data.iter().flat_map(|v| *v).collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        _ => unimplemented!(),
    };
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)?;
    let mut raw_assets = RawAssets::new();
    raw_assets.insert(path, bytes);
    Ok(raw_assets)
}

mod test {

    #[test]
    pub fn deserialize_png() {
        let tex: crate::Texture2D = crate::io::RawAssets::new()
            .insert(
                "test.png",
                include_bytes!("../../test_data/test.png").to_vec(),
            )
            .deserialize("")
            .unwrap();
        if let crate::TextureData::RgbaU8(data) = tex.data {
            assert_eq!(
                data,
                vec![
                    [0, 0, 0, 255],
                    [255, 0, 0, 255],
                    [0, 255, 0, 255],
                    [0, 0, 255, 255],
                ]
            );
        } else {
            panic!("Wrong texture data")
        }
        assert_eq!(tex.width, 2);
        assert_eq!(tex.height, 2);
    }

    #[test]
    pub fn serialize_png() {
        use crate::io::Serialize;
        let tex = crate::Texture2D {
            data: crate::TextureData::RgbaU8(vec![
                [0, 0, 0, 255],
                [255, 0, 0, 255],
                [0, 255, 0, 255],
                [0, 0, 255, 255],
            ]),
            width: 2,
            height: 2,
            ..Default::default()
        };
        let img = tex.serialize("test.png").unwrap();

        assert_eq!(
            include_bytes!("../../test_data/test.png"),
            img.get("test.png").unwrap()
        );
    }

    #[test]
    pub fn deserialize_hdr() {
        let tex: crate::Texture2D = crate::io::RawAssets::new()
            .insert(
                "test.hdr",
                include_bytes!("../../test_data/test.hdr").to_vec(),
            )
            .deserialize("")
            .unwrap();
        if let crate::TextureData::RgbF32(data) = tex.data {
            assert_eq!(data[0], [0.16503906, 0.24609375, 0.20019531]);
        } else {
            panic!("Wrong texture data")
        }
        assert_eq!(tex.width, 1024);
        assert_eq!(tex.height, 512);
    }
}