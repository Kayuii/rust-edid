use nom::{
    bytes::streaming::{tag, take},
    combinator::{map, not, peek},
    error::{context, VerboseError},
    multi::many0,
    number::streaming::le_u8,
    sequence::tuple,
    IResult,
};

use crate::edid::{parse_detailed_timing, DetailedTiming};

#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub struct NativeDTDs {
    pub underscan: u8,
    pub basic_audio: u8,
    pub ycbcr444: u8,
    pub ycbcr422: u8,
    pub number_of_native_dtd: u8,
}

fn parse_native_dtds(input: &[u8]) -> IResult<&[u8], NativeDTDs, VerboseError<&[u8]>> {
    let (input, v) = le_u8(input)?;
    Ok((
        input,
        NativeDTDs {
            underscan: (v & 0x80u8) >> 7,
            basic_audio: (v & 0x40u8) >> 6,
            ycbcr444: (v & 0x20u8) >> 5,
            ycbcr422: (v & 0x10u8) >> 4,
            number_of_native_dtd: v & 0xfu8,
        },
    ))
}

#[derive(Debug, PartialEq, Clone)]
pub struct DataBlockHeader {
    pub type_tag: u8,
    pub len: u8,
}

// impl fmt::Display for DataBlockHeader {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{:03b}{:05b}", self.type_tag, self.len)
//     }
// }

fn parse_data_block_header(input: &[u8]) -> IResult<&[u8], DataBlockHeader, VerboseError<&[u8]>> {
    map(le_u8, |v| DataBlockHeader {
        type_tag: (v & 0xe0u8) >> 5,
        len: v & 0x1fu8,
    })(input)
}

#[derive(Debug, PartialEq, Clone)]
pub enum DataBlock {
    Reserved(DataBlockReserved),
    AudioBlock(AudioBlock),
    VideoBlock(VideoBlock),
    VendorSpecific(VendorSpecific),
    SpeakerAllocation(SpeakerAllocation),
}

fn parse_blocks(input: &[u8]) -> IResult<&[u8], Vec<DataBlock>, VerboseError<&[u8]>> {
    many0(parse_data_block)(input)
}

#[derive(Debug, PartialEq, Clone)]
pub struct AudioBlock {
    pub header: DataBlockHeader,
    pub descriptors: Vec<ShortAudioDescriptor>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ShortAudioDescriptor {
    pub audio_format: u8,
    pub number_of_channels: u8,
    pub sampling_frequences: u8,
    pub format_dependent_value: u8,
    pub audio_format_extended_code: u8,
}

// pub enum AudioFormatCode {
//     RESERVED,
//     LPCM,
//     AC3,
//     MPEG1,
//     MP3,
//     MPEG2,
//     AAC,
//     DTS,
//     ATRAC,
//     DSD,
//     DDPLUS,
//     DTSHD,
//     TRUEHD,
//     DSTAUDIO,
//     WMAPRO,
//     EXTENSION,
// }

fn parse_audio_block(input: &[u8]) -> IResult<&[u8], AudioBlock, VerboseError<&[u8]>> {
    context("audio data blocks", |i| {
        let (i, header) = parse_data_block_header(i)?;
        let (i, payload) = take(header.len)(i)?;
        let (_i, descriptors) = many0(map(
            tuple((le_u8, le_u8, le_u8)),
            |(format_and_channels, sampling_frequences, bitrate_or_bitdepth)| {
                ShortAudioDescriptor {
                    audio_format: (format_and_channels & 0x78u8) >> 3,
                    number_of_channels: (format_and_channels & 0x7u8) + 1u8,
                    sampling_frequences,
                    audio_format_extended_code: (bitrate_or_bitdepth & 0xf8u8) >> 3,
                    format_dependent_value: bitrate_or_bitdepth & 0x7u8,
                }
            },
        ))(payload)?;
        Ok((
            i,
            AudioBlock {
                header,
                descriptors,
            },
        ))
    })(input)
}

#[derive(Debug, PartialEq, Clone)]
pub struct ShortVideoDescriptor {
    pub is_native: u8,
    pub cea861_index: u8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VideoBlock {
    pub header: DataBlockHeader,
    pub descriptors: Vec<ShortVideoDescriptor>,
}

fn parse_video_block(input: &[u8]) -> IResult<&[u8], VideoBlock, VerboseError<&[u8]>> {
    context("video data blocks", |i| {
        let (i, header) = parse_data_block_header(i)?;
        let (i, payload) = take(header.len)(i)?;
        let (_i, descriptors) = many0(map(le_u8, |payload| ShortVideoDescriptor {
            is_native: (payload & 0x80u8) >> 7,
            cea861_index: payload & 0x7fu8,
        }))(payload)?;
        Ok((
            i,
            VideoBlock {
                header,
                descriptors,
            },
        ))
    })(input)
}

#[derive(Debug, PartialEq, Clone)]
pub struct VendorSpecific {
    pub header: DataBlockHeader,
    pub identifier: [u8; 3],
    pub payload: Vec<u8>,
}

fn parse_vendor_specific(input: &[u8]) -> IResult<&[u8], VendorSpecific, VerboseError<&[u8]>> {
    context("vendor specific data block", |i| {
        let (i, header) = parse_data_block_header(i)?;
        let (i, payload) = take(header.len)(i)?;
        let (payload, identifier) = take(3u8)(payload)?;
        let (_i, payload) = take(header.len - 3)(payload)?;
        Ok((
            i,
            VendorSpecific {
                header,
                identifier: identifier.try_into().unwrap(),
                payload: Vec::from(payload), // payload 类型由编译器推断
            },
        ))
    })(input)
}

#[derive(Debug, PartialEq, Clone)]
pub struct SpeakerAllocation {
    pub header: DataBlockHeader,
    pub speakers: u8,
    pub reserved: [u8; 2],
}

impl SpeakerAllocation {
    pub const REAR_LEFT_RIGHT_CENTER: u8 = (1u8 << 6);
    pub const FRONT_LEFT_RIGHT_CENTER: u8 = (1u8 << 5);
    pub const REAR_CENTER: u8 = (1u8 << 4);
    pub const REAR_LEFT_RIGHT: u8 = (1u8 << 3);
    pub const FRONT_CENTER: u8 = (1u8 << 2);
    pub const LFE: u8 = (1u8 << 1);
    pub const FRONT_LEFT_RIGHT: u8 = (1u8 << 0);
}

fn parse_speaker_allocation(
    input: &[u8],
) -> IResult<&[u8], SpeakerAllocation, VerboseError<&[u8]>> {
    context("speaker allocation data block", |i| {
        let (i, header) = parse_data_block_header(i)?;
        let (i, payload) = take(header.len)(i)?;
        let (payload, speakers) = take(1u8)(payload)?;
        let (_i, reserved) = take(2u8)(payload)?;
        Ok((
            i,
            SpeakerAllocation {
                header,
                speakers: speakers[0],
                reserved: [reserved[0], reserved[1]],
            },
        ))
    })(input)
}

fn parse_data_block_reserved(
    input: &[u8],
) -> IResult<&[u8], DataBlockReserved, VerboseError<&[u8]>> {
    let (input, header) = parse_data_block_header(input)?;
    let (input, payload) = take(header.len)(input)?;

    Ok((
        input,
        DataBlockReserved {
            header,
            payload: payload.to_vec(),
        },
    ))
}

fn parse_data_block(input: &[u8]) -> IResult<&[u8], DataBlock, VerboseError<&[u8]>> {
    let (remaining, header) = peek(parse_data_block_header)(input)?;
    // println!("data block type: {:?}", header.type_tag);
    // println!("data block len: {:?}", header.len);
    match header.type_tag {
        0b001 => map(parse_audio_block, |v| DataBlock::AudioBlock(v))(remaining),
        0b010 => map(parse_video_block, |v| DataBlock::VideoBlock(v))(remaining),
        0b011 => map(parse_vendor_specific, |v| DataBlock::VendorSpecific(v))(remaining),
        0b100 => map(parse_speaker_allocation, |v| {
            DataBlock::SpeakerAllocation(v)
        })(remaining),
        // 0b101 => map(parse_audio_block, |v| DataBlock::AudioBlock(v))(input),
        // 0b110 => map(parse_audio_block, |v| DataBlock::AudioBlock(v))(input),
        // 0b111 => map(parse_audio_block, |v| DataBlock::AudioBlock(v))(input),
        // _ => Ok((
        //     remaining,
        //     DataBlock::Reserved(DataBlockReserved {
        //         header,
        //         payload: Vec::from(block_data),
        //     }),
        // )),
        _ => map(parse_data_block_reserved, |v| DataBlock::Reserved(v))(remaining),
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct DataBlockReserved {
    pub header: DataBlockHeader,
    pub payload: Vec<u8>,
}


#[derive(Debug, PartialEq, Clone, Default)]
pub struct CtaExtensions {
    pub extension_tag: u8,
    pub reserved: u8,
    pub native_dtd: NativeDTDs,
    pub blocks: Vec<DataBlock>,
    pub descriptors: Vec<DetailedTiming>,
}

impl CtaExtensions {
    // native DTD information bits
    pub const DTD_UNDERSCAN: u8 = (1u8 << 7); // display supports underscan
    pub const DTD_BASIC_AUDIO: u8 = (1u8 << 6); // display supports basic audio
    pub const DTD_YUV444: u8 = (1u8 << 5); // display supports YCbCr 4∶4∶4
    pub const DTD_YUV422: u8 = (1u8 << 4); // display supports YCbCr 4∶2∶2
}

fn parse_descriptors(input: &[u8]) -> IResult<&[u8], Vec<DetailedTiming>, VerboseError<&[u8]>> {    
    many0(map(
        tuple((
            peek(not(tag(&[0, 0]))),
            take(18u8),
        )),
        | (_, data)| {
            let (_, detailed_timing) =  parse_detailed_timing(data).unwrap();
            detailed_timing
        },
    ))(input)
}

pub(crate) fn parse_extension(input: &[u8]) -> IResult<&[u8], CtaExtensions, VerboseError<&[u8]>> {
    let (input, (extension_tag, reserved, dtd_flag)) = tuple((le_u8, le_u8, le_u8))(input)?;
    if dtd_flag == 0 {
        return Ok((
            &input[128..],
            CtaExtensions {
                extension_tag,
                reserved,
                blocks: Vec::new(),
                descriptors: Vec::new(),
                ..Default::default()
            },
        ));
    }
    println!("dtd_flag: {:?}", dtd_flag);

    let (input, native_dtd) = parse_native_dtds(input)?;
    let (input, extension_data) = take(dtd_flag - 4)(input)?;
    let (_, data_block) = parse_blocks(extension_data)?;
    let (input, detailed_timing_data) = take(input.len() as u8 -1 )(input)?;
    let (_, detailed_timing) = parse_descriptors(detailed_timing_data)?;

    let (input, _checksum) = le_u8(input)?;

    println!("input[{:b}]", _checksum);

    Ok((
        input,
        CtaExtensions {
            extension_tag,
            reserved,
            native_dtd,
            blocks: data_block,
            descriptors: detailed_timing,
            ..Default::default()
        },
    ))
}
