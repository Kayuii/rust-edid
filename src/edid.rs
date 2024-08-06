use nom::{
    bytes::complete::{tag, take},
    combinator::{map, peek},
    error::VerboseError,
    multi::count,
    number::complete::{be_u16, le_u16, le_u32, le_u8},
    sequence::{terminated, tuple},
    IResult,
};
use std::convert::TryInto;

use crate::{cp437, extension::{parse_extension, CtaExtensions}};

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
    terminated(
        map(
            tuple((
                tag(&[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00][..]),
                map(be_u16, parse_vendor),
                le_u16,
                le_u32,
                le_u8,
                le_u8,
                le_u8,
                le_u8,
            )),
            |(_, vendor, product, serial, week, year, version, revision)| Header {
                vendor,
                product,
                serial,
                week,
                year,
                version,
                revision,
            },
        ),
        take(0usize), // Consume any trailing bytes
    )(input)
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
    map(
        tuple((le_u8, le_u8, le_u8, le_u8, le_u8)),
        |(video_input, width, height, gamma, features)| Display {
            video_input,
            width,
            height,
            gamma,
            features,
        },
    )(input)
}

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

#[derive(Debug, PartialEq, Copy, Clone, Default)]
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

pub(crate) fn parse_detailed_timing(input: &[u8]) -> IResult<&[u8], DetailedTiming, VerboseError<&[u8]>> {
    map(
        tuple((
            le_u16, // pixel_clock_10khz
            le_u8,  // horizontal_active_lo
            le_u8,  // horizontal_blanking_lo
            le_u8,  // horizontal_px_hi
            le_u8,  // vertical_active_lo
            le_u8,  // vertical_blanking_lo
            le_u8,  // vertical_px_hi
            le_u8,  // horizontal_front_porch_lo
            le_u8,  // horizontal_sync_width_lo
            le_u8,  // vertical_lo
            le_u8,  // porch_sync_hi
            le_u8,  // horizontal_size_lo
            le_u8,  // vertical_size_lo
            le_u8,  // size_hi
            le_u8,  // horizontal_border
            le_u8,  // vertical_border
            le_u8,  // features
        )),
        |(
            pixel_clock_10khz,
            horizontal_active_lo,
            horizontal_blanking_lo,
            horizontal_px_hi,
            vertical_active_lo,
            vertical_blanking_lo,
            vertical_px_hi,
            horizontal_front_porch_lo,
            horizontal_sync_width_lo,
            vertical_lo,
            porch_sync_hi,
            horizontal_size_lo,
            vertical_size_lo,
            size_hi,
            horizontal_border,
            vertical_border,
            features,
        )| DetailedTiming {
            pixel_clock: pixel_clock_10khz as u32 * 10,
            horizontal_active_pixels: (horizontal_active_lo as u16)
                | (((horizontal_px_hi >> 4) as u16) << 8),
            horizontal_blanking_pixels: (horizontal_blanking_lo as u16)
                | (((horizontal_px_hi & 0xf) as u16) << 8),
            vertical_active_lines: (vertical_active_lo as u16)
                | (((vertical_px_hi >> 4) as u16) << 8),
            vertical_blanking_lines: (vertical_blanking_lo as u16)
                | (((vertical_px_hi & 0xf) as u16) << 8),
            horizontal_front_porch: (horizontal_front_porch_lo as u16)
                | (((porch_sync_hi >> 6) as u16) << 8),
            horizontal_sync_width: (horizontal_sync_width_lo as u16)
                | ((((porch_sync_hi >> 4) & 0x3) as u16) << 8),
            vertical_front_porch: ((vertical_lo >> 4) as u16)
                | ((((porch_sync_hi >> 2) & 0x3) as u16) << 8),
            vertical_sync_width: ((vertical_lo & 0xf) as u16)
                | (((porch_sync_hi & 0x3) as u16) << 8),
            horizontal_size: (horizontal_size_lo as u16) | (((size_hi >> 4) as u16) << 8),
            vertical_size: (vertical_size_lo as u16) | (((size_hi & 0xf) as u16) << 8),
            horizontal_border_pixels: horizontal_border,
            vertical_border_pixels: vertical_border,
            features,
        },
    )(input)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Descriptor {
    DetailedTiming(DetailedTiming),
    SerialNumber(String),
    UnspecifiedText(String),
    RangeLimits,
    ProductName(String),
    WhitePoint,
    StandardTiming,
    ColorManagement,
    TimingCodes,
    EstablishedTimings,
    Dummy,
    Unknown([u8; 13]),
}

fn parse_descriptor(input: &[u8]) -> IResult<&[u8], Descriptor, VerboseError<&[u8]>> {
    let (remaining, peeked) = peek(le_u16)(input)?;
    match peeked {
        0 => {
            let (remaining, _) = take(3u8)(remaining)?;
            let (remaining, discriminant) = le_u8(remaining)?;
            let (remaining, _) = le_u8(remaining)?;

            match discriminant {
                0xFF => map(parse_descriptor_text, |s| Descriptor::SerialNumber(s))(remaining),
                0xFE => map(parse_descriptor_text, |s| Descriptor::UnspecifiedText(s))(remaining),
                0xFD => map(take(13u8), |_discarded: &[u8]| Descriptor::RangeLimits)(remaining),
                0xFC => map(parse_descriptor_text, |s| Descriptor::ProductName(s))(remaining),
                0xFB => map(take(13u8), |_discarded: &[u8]| Descriptor::WhitePoint)(remaining),
                0xFA => map(take(13u8), |_discarded: &[u8]| Descriptor::StandardTiming)(remaining),
                0xF9 => map(take(13u8), |_discarded: &[u8]| Descriptor::ColorManagement)(remaining),
                0xF8 => map(take(13u8), |_discarded: &[u8]| Descriptor::TimingCodes)(remaining),
                0xF7 => map(take(13u8), |_discarded: &[u8]| {
                    Descriptor::EstablishedTimings
                })(remaining),
                0x10 => map(take(13u8), |_discarded: &[u8]| Descriptor::Dummy)(remaining),
                _ => map(take(13u8), |data: &[u8]| {
                    Descriptor::Unknown(data.try_into().unwrap())
                })(remaining),
            }
        }
        _ => {
            let (remaining, detailed_timing) = parse_detailed_timing(remaining)?;
            Ok((remaining, Descriptor::DetailedTiming(detailed_timing)))
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct EDID {
    pub header: Header,
    pub display: Display,
    pub chromaticity: (),       
    pub established_timing: (), 
    pub standard_timing: (),    
    pub descriptors: Vec<Descriptor>,
    pub extensions: Option<CtaExtensions>,

}

fn parse_edid(input: &[u8]) -> IResult<&[u8], EDID, VerboseError<&[u8]>> {
    let (input, (
        header,
        display,
        chromaticity,
        established_timing,
        standard_timing,
        descriptors,
        number_of_extensions,
        _checksum
    )) = tuple((
        parse_header,
        parse_display,
        parse_chromaticity,
        parse_established_timing,
        parse_standard_timing,
        map(count(parse_descriptor, 4), Vec::from),
        le_u8,
        le_u8,
    ))(input)?;

    if number_of_extensions == 0 {
        return Ok((input, EDID {
            header,
            display,
            chromaticity,
            established_timing,
            standard_timing,
            descriptors,
            extensions: None,
        }));
    }

    // let (input, extensions) = map(
    //     count(move |input| parse_extension(input), number_of_extensions as usize),
    //     Vec::from,
    // )(input)?;
    let (input, extensions) = parse_extension(input)?;

    Ok((
        input,
        EDID {
            header,
            display,
            chromaticity,
            established_timing,
            standard_timing,
            descriptors,
            extensions: Some(extensions),
        },
    ))
}

pub fn parse(data: &[u8]) -> nom::IResult<&[u8], EDID, VerboseError<&[u8]>> {
    parse_edid(data)
}
