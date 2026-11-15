//! Recognises the image formats moodboards accept (PNG, JPEG, GIF, WebP) from
//! their first bytes and reads their size, without decoding the picture. All
//! three system web views display these formats.

/// Largest image a moodboard accepts.
pub const MAX_BYTES: usize = 50 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageInfo {
    /// File extension to store it under.
    pub extension: &'static str,
    /// As displayed, i.e. after a JPEG's EXIF orientation is applied.
    pub width: u32,
    pub height: u32,
}

/// Content type for serving a stored image, from its extension.
pub fn content_type(file: &str) -> Option<&'static str> {
    let extension = file.rsplit_once('.')?.1.to_ascii_lowercase();
    match extension.as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

/// Format and size of an image, or None if it is none of the accepted ones.
pub fn inspect(bytes: &[u8]) -> Option<ImageInfo> {
    let info = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        png(bytes)
    } else if bytes.starts_with(&[0xFF, 0xD8]) {
        jpeg(bytes)
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        gif(bytes)
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        webp(bytes)
    } else {
        None
    }?;
    (info.width > 0 && info.height > 0).then_some(info)
}

fn u16_be(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u16::from_be_bytes(bytes.get(at..at + 2)?.try_into().ok()?) as u32)
}

fn u16_le(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?) as u32)
}

fn u24_le(bytes: &[u8], at: usize) -> Option<u32> {
    let b = bytes.get(at..at + 3)?;
    Some(b[0] as u32 | (b[1] as u32) << 8 | (b[2] as u32) << 16)
}

fn u32_be(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

fn png(bytes: &[u8]) -> Option<ImageInfo> {
    // The first chunk is always IHDR: width and height at offsets 16 and 20.
    if bytes.get(12..16)? != b"IHDR" {
        return None;
    }
    Some(ImageInfo {
        extension: "png",
        width: u32_be(bytes, 16)?,
        height: u32_be(bytes, 20)?,
    })
}

fn gif(bytes: &[u8]) -> Option<ImageInfo> {
    Some(ImageInfo {
        extension: "gif",
        width: u16_le(bytes, 6)?,
        height: u16_le(bytes, 8)?,
    })
}

fn webp(bytes: &[u8]) -> Option<ImageInfo> {
    let (width, height) = match bytes.get(12..16)? {
        b"VP8 " => {
            // Key frame start code, then 14-bit width and height.
            if bytes.get(23..26)? != [0x9D, 0x01, 0x2A] {
                return None;
            }
            (u16_le(bytes, 26)? & 0x3FFF, u16_le(bytes, 28)? & 0x3FFF)
        }
        b"VP8L" => {
            if *bytes.get(20)? != 0x2F {
                return None;
            }
            let bits = u32::from_le_bytes(bytes.get(21..25)?.try_into().ok()?);
            ((bits & 0x3FFF) + 1, ((bits >> 14) & 0x3FFF) + 1)
        }
        b"VP8X" => (u24_le(bytes, 24)? + 1, u24_le(bytes, 27)? + 1),
        _ => return None,
    };
    Some(ImageInfo {
        extension: "webp",
        width,
        height,
    })
}

fn jpeg(bytes: &[u8]) -> Option<ImageInfo> {
    let mut at = 2;
    let mut swapped = false;
    loop {
        // Markers may be padded with extra 0xFF bytes.
        while *bytes.get(at)? == 0xFF && *bytes.get(at + 1)? == 0xFF {
            at += 1;
        }
        if *bytes.get(at)? != 0xFF {
            return None;
        }
        let marker = *bytes.get(at + 1)?;
        let length = u16_be(bytes, at + 2)? as usize;
        if length < 2 {
            return None;
        }
        let segment = bytes.get(at + 4..at + 2 + length);
        match marker {
            // APP1 with EXIF: phones store photos sideways plus an
            // orientation that the web views apply when showing them.
            0xE1 => {
                if let Some(orientation) = segment.and_then(exif_orientation) {
                    swapped = (5..=8).contains(&orientation);
                }
            }
            // Start of frame (all kinds except DHT, JPG and DAC).
            0xC0..=0xCF if !matches!(marker, 0xC4 | 0xC8 | 0xCC) => {
                let height = u16_be(bytes, at + 5)?;
                let width = u16_be(bytes, at + 7)?;
                let (width, height) = if swapped {
                    (height, width)
                } else {
                    (width, height)
                };
                return Some(ImageInfo {
                    extension: "jpg",
                    width,
                    height,
                });
            }
            // Start of scan without a frame before it: broken.
            0xDA => return None,
            _ => {}
        }
        at += 2 + length;
    }
}

/// The orientation tag (1–8) from an APP1 segment, if it has one.
fn exif_orientation(segment: &[u8]) -> Option<u32> {
    let tiff = segment.strip_prefix(b"Exif\0\0")?;
    let big_endian = match tiff.get(..2)? {
        b"MM" => true,
        b"II" => false,
        _ => return None,
    };
    let read16 = |at: usize| {
        if big_endian {
            u16_be(tiff, at)
        } else {
            u16_le(tiff, at)
        }
    };
    let read32 = |at: usize| {
        let b: [u8; 4] = tiff.get(at..at + 4)?.try_into().ok()?;
        Some(if big_endian {
            u32::from_be_bytes(b)
        } else {
            u32::from_le_bytes(b)
        })
    };
    let ifd = read32(4)? as usize;
    let count = read16(ifd)? as usize;
    (0..count).find_map(|index| {
        let entry = ifd + 2 + index * 12;
        (read16(entry)? == 0x0112).then(|| read16(entry + 8))?
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A minimal PNG header of the given size, enough for `inspect`.
    pub fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        bytes.extend(width.to_be_bytes());
        bytes.extend(height.to_be_bytes());
        bytes.extend([8, 6, 0, 0, 0]);
        bytes
    }

    fn jpeg_bytes(width: u16, height: u16, orientation: Option<u16>) -> Vec<u8> {
        let mut bytes = vec![0xFF, 0xD8];
        // APP0 (JFIF) to skip.
        bytes.extend([0xFF, 0xE0, 0x00, 0x07, b'J', b'F', b'I', b'F', 0]);
        if let Some(orientation) = orientation {
            let mut tiff = b"II\x2a\0\x08\0\0\0".to_vec();
            tiff.extend(1u16.to_le_bytes());
            tiff.extend(0x0112u16.to_le_bytes());
            tiff.extend(3u16.to_le_bytes());
            tiff.extend(1u32.to_le_bytes());
            tiff.extend(orientation.to_le_bytes());
            tiff.extend([0, 0]);
            let mut segment = b"Exif\0\0".to_vec();
            segment.extend(tiff);
            bytes.extend([0xFF, 0xE1]);
            bytes.extend(((segment.len() + 2) as u16).to_be_bytes());
            bytes.extend(segment);
        }
        bytes.extend([0xFF, 0xFF, 0xC2, 0x00, 0x11, 0x08]);
        bytes.extend(height.to_be_bytes());
        bytes.extend(width.to_be_bytes());
        bytes.extend([0; 10]);
        bytes
    }

    fn info(extension: &'static str, width: u32, height: u32) -> Option<ImageInfo> {
        Some(ImageInfo {
            extension,
            width,
            height,
        })
    }

    #[test]
    fn reads_png_and_gif_sizes() {
        assert_eq!(inspect(&png_bytes(640, 480)), info("png", 640, 480));
        let mut gif = b"GIF89a".to_vec();
        gif.extend([0x20, 0x03, 0x58, 0x02, 0, 0, 0]);
        assert_eq!(inspect(&gif), info("gif", 800, 600));
    }

    #[test]
    fn reads_jpeg_sizes_and_applies_the_orientation() {
        assert_eq!(
            inspect(&jpeg_bytes(4032, 3024, None)),
            info("jpg", 4032, 3024)
        );
        assert_eq!(
            inspect(&jpeg_bytes(4032, 3024, Some(1))),
            info("jpg", 4032, 3024)
        );
        assert_eq!(
            inspect(&jpeg_bytes(4032, 3024, Some(6))),
            info("jpg", 3024, 4032)
        );
    }

    #[test]
    fn reads_all_three_webp_kinds() {
        let riff = |kind: &[u8], rest: &[u8]| {
            let mut bytes = b"RIFF\0\0\0\0WEBP".to_vec();
            bytes.extend(kind);
            bytes.extend([0, 0, 0, 0]);
            bytes.extend(rest);
            bytes
        };
        let lossy = riff(
            b"VP8 ",
            &[0, 0, 0, 0x9D, 0x01, 0x2A, 0x90, 0x01, 0x2C, 0x01],
        );
        assert_eq!(inspect(&lossy), info("webp", 400, 300));
        // 14 bits width - 1 = 399, then 14 bits height - 1 = 299.
        let bits: u32 = 399 | (299 << 14);
        let mut lossless = vec![0x2F];
        lossless.extend(bits.to_le_bytes());
        assert_eq!(inspect(&riff(b"VP8L", &lossless)), info("webp", 400, 300));
        let extended = riff(b"VP8X", &[0, 0, 0, 0, 0x8F, 0x01, 0, 0x2B, 0x01, 0]);
        assert_eq!(inspect(&extended), info("webp", 400, 300));
    }

    #[test]
    fn rejects_other_and_broken_files() {
        assert_eq!(inspect(b""), None);
        assert_eq!(inspect(b"%PDF-1.7"), None);
        assert_eq!(
            inspect(b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>"),
            None
        );
        assert_eq!(inspect(&png_bytes(640, 480)[..20]), None);
        assert_eq!(inspect(&png_bytes(0, 480)), None);
        assert_eq!(inspect(&jpeg_bytes(100, 100, None)[..12]), None);
        assert_eq!(inspect(&[0xFF, 0xD8, 0xFF, 0xDA, 0x00, 0x02]), None);
    }

    #[test]
    fn content_types_follow_the_extension() {
        assert_eq!(content_type("01J.PNG"), Some("image/png"));
        assert_eq!(content_type("a.jpeg"), Some("image/jpeg"));
        assert_eq!(content_type("a.webp"), Some("image/webp"));
        assert_eq!(content_type("a.svg"), None);
        assert_eq!(content_type("ohne"), None);
    }
}
