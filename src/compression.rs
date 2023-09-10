use std::io::{self, BufReader, Cursor, Read, Write};

use bytes::Buf;

pub enum Compressor {
    Zstd,
    Webp,
}

impl Compressor {
    pub fn new(mime: &str) -> Compressor {
        match mime {
            _ if mime.contains("image") => Compressor::Webp,
            _ => Compressor::Zstd,
        }
    }
}

pub trait Compression {
    fn compress<T: Read, Q: Write>(&self, data: T, dest: &mut Q) -> io::Result<()>;
    fn decompress<T: Read, Q: Write>(&self, data: T, dest: &mut Q) -> io::Result<()>;
}

impl Compression for Compressor {
    fn compress<T: Read, Q: Write>(&self, data: T, dest: &mut Q) -> io::Result<()> {
        match self {
            Compressor::Zstd => zstd::stream::copy_encode(data, dest, 0),
            Compressor::Webp => convert_into_webp(data, dest),
        }
    }

    fn decompress<T: Read, Q: Write>(&self, data: T, dest: &mut Q) -> io::Result<()> {
        match self {
            Compressor::Zstd => zstd::stream::copy_decode(data, dest),
            Compressor::Webp => {
                io::copy(&mut BufReader::new(data), dest)?;
                Ok(())
            }
        }
    }
}

fn convert_into_webp<T: Read, Q: Write>(mut data: T, dest: &mut Q) -> io::Result<()> {
    let mut buf = vec![];
    let _ = data.read_to_end(&mut buf)?;
    let image = image::io::Reader::new(Cursor::new(buf))
        .with_guessed_format()?
        .decode()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let encoder = webp::Encoder::from_image(&image)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let binding = encoder.encode(65f32);
    std::io::copy(&mut binding.reader(), dest)?;
    Ok(())
}
