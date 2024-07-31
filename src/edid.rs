use nom::{
    bytes::streaming::{tag, take},
    combinator::{map, peek},
    error::VerboseError,
    multi::count,
    number::streaming::{be_u16, le_u16, le_u32, le_u8},
    IResult,
};
use std::convert::TryInto;

use crate::cp437;


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
    let (remaining, _) = tag(&[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00][..])(input)?;
    let (remaining, vendor) = map(be_u16, parse_vendor)(remaining)?;
    let (remaining, product) = le_u16(remaining)?;
    let (remaining, serial) = le_u32(remaining)?;
    let (remaining, week) = le_u8(remaining)?;
    let (remaining, year) = le_u8(remaining)?;
    let (remaining, version) = le_u8(remaining)?;
    let (remaining, revision) = le_u8(remaining)?;
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
    let (remaining, video_input) = le_u8(input)?;
    let (remaining, width) = le_u8(remaining)?;
    let (remaining, height) = le_u8(remaining)?;
    let (remaining, gamma) = le_u8(remaining)?;
    let (remaining, features) = le_u8(remaining)?;
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
    let (remaining, pixel_clock_10khz) = le_u16(input)?;
    let (remaining, horizontal_active_lo) = le_u8(remaining)?;
    let (remaining, horizontal_blanking_lo) = le_u8(remaining)?;
    let (remaining, horizontal_px_hi) = le_u8(remaining)?;
    let (remaining, vertical_active_lo) = le_u8(remaining)?;
    let (remaining, vertical_blanking_lo) = le_u8(remaining)?;
    let (remaining, vertical_px_hi) = le_u8(remaining)?;
    let (remaining, horizontal_front_porch_lo) = le_u8(remaining)?;
    let (remaining, horizontal_sync_width_lo) = le_u8(remaining)?;
    let (remaining, vertical_lo) = le_u8(remaining)?;
    let (remaining, porch_sync_hi) = le_u8(remaining)?;
    let (remaining, horizontal_size_lo) = le_u8(remaining)?;
    let (remaining, vertical_size_lo) = le_u8(remaining)?;
    let (remaining, size_hi) = le_u8(remaining)?;
    let (remaining, horizontal_border) = le_u8(remaining)?;
    let (remaining, vertical_border) = le_u8(remaining)?;
    let (remaining, features) = le_u8(remaining)?;

    Ok((
        remaining,
        DetailedTiming {
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
    ))
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
    pub chromaticity: (),       // TODO
    pub established_timing: (), // TODO
    pub standard_timing: (),    // TODO
    pub descriptors: Vec<Descriptor>,
}

fn parse_edid(input: &[u8]) -> IResult<&[u8], EDID, VerboseError<&[u8]>> {
    let (remaining, header) = parse_header(input)?;
    let (remaining, display) = parse_display(remaining)?;
    let (remaining, chromaticity) = parse_chromaticity(remaining)?;
    let (remaining, established_timing) = parse_established_timing(remaining)?;
    let (remaining, standard_timing) = parse_standard_timing(remaining)?;
    let (remaining, descriptors) = count(parse_descriptor, 4)(remaining)?;
    let (remaining, _number_of_extensions) = take(1u8)(remaining)?;
    let (remaining, _checksum) = take(1u8)(remaining)?;
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
