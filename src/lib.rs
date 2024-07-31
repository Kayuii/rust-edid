use nom::{
    bytes::streaming::{tag, take},
    combinator::{map, peek},
    error::VerboseError,
    multi::count,
    number::streaming::{be_u16, le_u16, le_u32, le_u8},
    IResult,
};
use std::convert::TryInto;

pub mod reexport {
    pub use nom;
}

mod cp437;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Header {
    pub vendor: [char; 3],
    pub product: u16,
    pub serial: u32,
    pub week: u8,
    pub year: u8, // Starting at year 1990
    pub version: u8,
    pub revision: u8,
}

fn parse_vendor(v: u16) -> [char; 3] {
    let mask: u8 = 0x1F; // Each letter is 5 bits
    let i0 = ('A' as u8) - 1; // 0x01 = A
    return [
        (((v >> 10) as u8 & mask) + i0) as char,
        (((v >> 5) as u8 & mask) + i0) as char,
        (((v >> 0) as u8 & mask) + i0) as char,
    ];
}

fn parse_header(input: &[u8]) -> IResult<&[u8], Header, VerboseError<&[u8]>> {
    let mut remaining = input;

    let _tag;
    (remaining, _tag) = tag(&[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00][..])(remaining)?;

    let vendor;
    (remaining, vendor) = map(be_u16, parse_vendor)(remaining)?;

    let product;
    (remaining, product) = le_u16(remaining)?;

    let serial;
    (remaining, serial) = le_u32(remaining)?;

    let week;
    (remaining, week) = le_u8(remaining)?;

    let year;
    (remaining, year) = le_u8(remaining)?;

    let version;
    (remaining, version) = le_u8(remaining)?;

    let revision;
    (remaining, revision) = le_u8(remaining)?;

    Ok((
        remaining,
        Header {
            vendor,
            product,
            serial,
            week,
            year,
            version,
            revision,
        },
    ))
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Display {
    pub video_input: u8,
    pub width: u8,  // cm
    pub height: u8, // cm
    pub gamma: u8,  // datavalue = (gamma*100)-100 (range 1.00–3.54)
    pub features: u8,
}

fn parse_display(input: &[u8]) -> IResult<&[u8], Display, VerboseError<&[u8]>> {
    let mut remaining = input;

    let video_input;
    (remaining, video_input) = le_u8(remaining)?;

    let width;
    (remaining, width) = le_u8(remaining)?;

    let height;
    (remaining, height) = le_u8(remaining)?;

    let gamma;
    (remaining, gamma) = le_u8(remaining)?;

    let features;
    (remaining, features) = le_u8(remaining)?;

    Ok((
        remaining,
        Display {
            video_input,
            width,
            height,
            gamma,
            features,
        },
    ))
}
// named!(parse_display<&[u8], Display, VerboseError<&[u8]>>, do_parse!(
//     video_input: le_u8
//     >> width: le_u8
//     >> height: le_u8
//     >> gamma: le_u8
//     >> features: le_u8
//     >> (Display{video_input, width, height, gamma, features})
// ));

fn parse_chromaticity(input: &[u8]) -> IResult<&[u8], (), VerboseError<&[u8]>> {
    map(take(10u8), |_bytes| ())(input)
}

fn parse_established_timing(input: &[u8]) -> IResult<&[u8], (), VerboseError<&[u8]>> {
    map(take(3u8), |_bytes| ())(input)
}

fn parse_standard_timing(input: &[u8]) -> IResult<&[u8], (), VerboseError<&[u8]>> {
    map(take(16u8), |_bytes| ())(input)
}

fn parse_descriptor_text(input: &[u8]) -> IResult<&[u8], String, VerboseError<&[u8]>> {
    map(take(13u8), |b: &[u8]| {
        b.iter()
            .filter(|c| **c != 0x0A)
            .map(|b| cp437::forward(*b))
            .collect::<String>()
            .trim()
            .to_string()
    })(input)
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct DetailedTiming {
    /// Pixel clock in kHz.
    pub pixel_clock: u32,
    pub horizontal_active_pixels: u16,
    pub horizontal_blanking_pixels: u16,
    pub vertical_active_lines: u16,
    pub vertical_blanking_lines: u16,
    pub horizontal_front_porch: u16,
    pub horizontal_sync_width: u16,
    pub vertical_front_porch: u16,
    pub vertical_sync_width: u16,
    /// Horizontal size in millimeters
    pub horizontal_size: u16,
    /// Vertical size in millimeters
    pub vertical_size: u16,
    /// Border pixels on one side of screen (i.e. total number is twice this)
    pub horizontal_border_pixels: u8,
    /// Border pixels on one side of screen (i.e. total number is twice this)
    pub vertical_border_pixels: u8,
    pub features: u8, /* TODO add enums etc. */
}

fn parse_detailed_timing(input: &[u8]) -> IResult<&[u8], DetailedTiming, VerboseError<&[u8]>> {
    let mut remaining = input;

    let pixel_clock = {
        let pixel_clock_10khz;
        (remaining, pixel_clock_10khz) = le_u16(remaining)?;
        u32::from(pixel_clock_10khz) * 10
    };

    let horizontal_active_pixels;
    let horizontal_blanking_pixels;
    {
        let horizontal_active_lo;
        (remaining, horizontal_active_lo) = le_u8(remaining)?;

        let horizontal_blanking_lo;
        (remaining, horizontal_blanking_lo) = le_u8(remaining)?;

        let horizontal_px_hi;
        (remaining, horizontal_px_hi) = le_u8(remaining)?;

        horizontal_active_pixels =
            (horizontal_active_lo as u16) | (((horizontal_px_hi >> 4) as u16) << 8);

        horizontal_blanking_pixels =
            (horizontal_blanking_lo as u16) | (((horizontal_px_hi & 0xf) as u16) << 8);
    }

    let vertical_active_lines;
    let vertical_blanking_lines;
    {
        let vertical_active_lo;
        (remaining, vertical_active_lo) = le_u8(remaining)?;

        let vertical_blanking_lo;
        (remaining, vertical_blanking_lo) = le_u8(remaining)?;

        let vertical_px_hi;
        (remaining, vertical_px_hi) = le_u8(remaining)?;

        vertical_active_lines = (vertical_active_lo as u16) | (((vertical_px_hi >> 4) as u16) << 8);

        vertical_blanking_lines =
            (vertical_blanking_lo as u16) | (((vertical_px_hi & 0xf) as u16) << 8);
    }

    let horizontal_front_porch;
    let horizontal_sync_width;
    let vertical_front_porch;
    let vertical_sync_width;
    {
        let horizontal_front_porch_lo;
        (remaining, horizontal_front_porch_lo) = le_u8(remaining)?;

        let horizontal_sync_width_lo;
        (remaining, horizontal_sync_width_lo) = le_u8(remaining)?;

        let vertical_lo;
        (remaining, vertical_lo) = le_u8(remaining)?;

        let porch_sync_hi;
        (remaining, porch_sync_hi) = le_u8(remaining)?;

        horizontal_front_porch =
            (horizontal_front_porch_lo as u16) | (((porch_sync_hi >> 6) as u16) << 8);
        horizontal_sync_width =
            (horizontal_sync_width_lo as u16) | ((((porch_sync_hi >> 4) & 0x3) as u16) << 8);
        vertical_front_porch =
            ((vertical_lo >> 4) as u16) | ((((porch_sync_hi >> 2) & 0x3) as u16) << 8);
        vertical_sync_width = ((vertical_lo & 0xf) as u16) | (((porch_sync_hi & 0x3) as u16) << 8);
    }

    let horizontal_size;
    let vertical_size;
    {
        let horizontal_size_lo;
        (remaining, horizontal_size_lo) = le_u8(remaining)?;

        let vertical_size_lo;
        (remaining, vertical_size_lo) = le_u8(remaining)?;

        let size_hi;
        (remaining, size_hi) = le_u8(remaining)?;

        horizontal_size = (horizontal_size_lo as u16) | (((size_hi >> 4) as u16) << 8);
        vertical_size = (vertical_size_lo as u16) | (((size_hi & 0xf) as u16) << 8);
    }

    let horizontal_border_pixels;
    (remaining, horizontal_border_pixels) = le_u8(remaining)?;

    let vertical_border_pixels;
    (remaining, vertical_border_pixels) = le_u8(remaining)?;

    let features;
    (remaining, features) = le_u8(remaining)?;

    Ok((
        remaining,
        DetailedTiming {
            pixel_clock,
            horizontal_active_pixels,
            horizontal_blanking_pixels,
            vertical_active_lines,
            vertical_blanking_lines,
            horizontal_front_porch,
            horizontal_sync_width,
            vertical_front_porch,
            vertical_sync_width,
            horizontal_size,
            vertical_size,
            horizontal_border_pixels,
            vertical_border_pixels,
            features,
        },
    ))
}

#[derive(Debug, PartialEq, Clone)]
pub enum Descriptor {
    DetailedTiming(DetailedTiming),
    SerialNumber(String),
    UnspecifiedText(String),
    RangeLimits, // TODO
    ProductName(String),
    WhitePoint,     // TODO
    StandardTiming, // TODO
    ColorManagement,
    TimingCodes,
    EstablishedTimings,
    Dummy,
    Unknown([u8; 13]),
}

fn parse_descriptor(input: &[u8]) -> IResult<&[u8], Descriptor, VerboseError<&[u8]>> {
    let mut remaining = input;

    let peeked;
    (remaining, peeked) = peek(le_u16)(remaining)?;

    match peeked {
        0 => {
            let _discarded; // TODO: What is this? Is this useful?
            (remaining, _discarded) = take(3u8)(remaining)?;

            let discriminant; // TODO: Describe this better! Why is this a discriminant?
            (remaining, discriminant) = le_u8(remaining)?;

            let _discarded; // TODO: What is this? Is this useful?
            (remaining, _discarded) = le_u8(remaining)?;

            match discriminant {
                0xFF => map(parse_descriptor_text, |s| Descriptor::SerialNumber(s))(remaining),
                0xFE => map(parse_descriptor_text, |s| Descriptor::UnspecifiedText(s))(remaining),
                // TODO: Parse this maybe?
                0xFD => map(take(13u8), |_discarded: &[u8]| Descriptor::RangeLimits)(remaining),
                0xFC => map(parse_descriptor_text, |s| Descriptor::ProductName(s))(remaining),
                // TODO: Parse this maybe?
                0xFB => map(take(13u8), |_discarded: &[u8]| Descriptor::WhitePoint)(remaining),
                // TODO: Parse this maybe?
                0xFA => map(take(13u8), |_discarded: &[u8]| Descriptor::StandardTiming)(remaining),
                // TODO: Parse this maybe?
                0xF9 => map(take(13u8), |_discarded: &[u8]| Descriptor::ColorManagement)(remaining),
                // TODO: Parse this maybe?
                0xF8 => map(take(13u8), |_discarded: &[u8]| Descriptor::TimingCodes)(remaining),
                // TODO: Parse this maybe?
                0xF7 => map(take(13u8), |_discarded: &[u8]| {
                    Descriptor::EstablishedTimings
                })(remaining),
                // TODO: Parse this maybe?
                0x10 => map(take(13u8), |_discarded: &[u8]| Descriptor::Dummy)(remaining),
                _ => map(take(13u8), |data: &[u8]| {
                    Descriptor::Unknown(data.try_into().unwrap())
                })(remaining),
            }
        }
        _ => {
            let detailed_timing;
            (remaining, detailed_timing) = parse_detailed_timing(remaining)?;
            Ok((remaining, Descriptor::DetailedTiming(detailed_timing)))
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct EDID {
    pub header: Header,
    pub display: Display,
    chromaticity: (),       // TODO
    established_timing: (), // TODO
    standard_timing: (),    // TODO
    pub descriptors: Vec<Descriptor>,
}

fn parse_edid(input: &[u8]) -> IResult<&[u8], EDID, VerboseError<&[u8]>> {
    let mut remaining = input;

    let header;
    (remaining, header) = parse_header(remaining)?;

    let display;
    (remaining, display) = parse_display(remaining)?;

    let chromaticity;
    (remaining, chromaticity) = parse_chromaticity(remaining)?;

    let established_timing;
    (remaining, established_timing) = parse_established_timing(remaining)?;

    let standard_timing;
    (remaining, standard_timing) = parse_standard_timing(remaining)?;

    let descriptors;
    (remaining, descriptors) = count(parse_descriptor, 4)(remaining)?;

    let _number_of_extensions;
    (remaining, _number_of_extensions) = take(1u8)(remaining)?;

    let _checksum;
    (remaining, _checksum) = take(1u8)(remaining)?;

    Ok((
        remaining,
        EDID {
            header,
            display,
            chromaticity,
            established_timing,
            standard_timing,
            descriptors,
        },
    ))
}

pub fn parse(data: &[u8]) -> nom::IResult<&[u8], EDID, VerboseError<&[u8]>> {
    parse_edid(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test(d: &[u8], expected: &EDID) {
        match parse(d) {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining.len(), 0);
                assert_eq!(&parsed, expected);
            }
            Err(err) => {
                panic!("{}", err);
            }
        }
    }

    #[test]
    fn test_card0_vga_1() {
        let d = include_bytes!("../testdata/card0-VGA-1");

        let expected = EDID {
            header: Header {
                vendor: ['S', 'A', 'M'],
                product: 596,
                serial: 1146106418,
                week: 27,
                year: 17,
                version: 1,
                revision: 3,
            },
            display: Display {
                video_input: 14,
                width: 47,
                height: 30,
                gamma: 120,
                features: 42,
            },
            chromaticity: (),
            established_timing: (),
            standard_timing: (),
            descriptors: vec![
                Descriptor::DetailedTiming(DetailedTiming {
                    pixel_clock: 146250,
                    horizontal_active_pixels: 1680,
                    horizontal_blanking_pixels: 560,
                    vertical_active_lines: 1050,
                    vertical_blanking_lines: 39,
                    horizontal_front_porch: 104,
                    horizontal_sync_width: 176,
                    vertical_front_porch: 3,
                    vertical_sync_width: 6,
                    horizontal_size: 474,
                    vertical_size: 296,
                    horizontal_border_pixels: 0,
                    vertical_border_pixels: 0,
                    features: 28,
                }),
                Descriptor::RangeLimits,
                Descriptor::ProductName("SyncMaster".to_string()),
                Descriptor::SerialNumber("HS3P701105".to_string()),
            ],
        };

        test(d, &expected);
    }

    #[test]
    fn test_card0_edp_1() {
        let d = include_bytes!("../testdata/card0-eDP-1");

        let expected = EDID {
            header: Header {
                vendor: ['S', 'H', 'P'],
                product: 5193,
                serial: 0,
                week: 32,
                year: 25,
                version: 1,
                revision: 4,
            },
            display: Display {
                video_input: 165,
                width: 29,
                height: 17,
                gamma: 120,
                features: 14,
            },
            chromaticity: (),
            established_timing: (),
            standard_timing: (),
            descriptors: vec![
                Descriptor::DetailedTiming(DetailedTiming {
                    pixel_clock: 138500,
                    horizontal_active_pixels: 1920,
                    horizontal_blanking_pixels: 160,
                    vertical_active_lines: 1080,
                    vertical_blanking_lines: 31,
                    horizontal_front_porch: 48,
                    horizontal_sync_width: 32,
                    vertical_front_porch: 3,
                    vertical_sync_width: 5,
                    horizontal_size: 294,
                    vertical_size: 165,
                    horizontal_border_pixels: 0,
                    vertical_border_pixels: 0,
                    features: 24,
                }),
                Descriptor::Dummy,
                Descriptor::UnspecifiedText("DJCP6ÇLQ133M1".to_string()),
                Descriptor::Unknown([2, 65, 3, 40, 0, 18, 0, 0, 11, 1, 10, 32, 32]),
            ],
        };

        test(d, &expected);
    }
}
