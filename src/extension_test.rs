#[cfg(test)]
mod tests {
    use crate::{edid::*, extension::*};

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
    fn test_card0_hdmi_1() {
        let d = include_bytes!("../testdata/card0-HDMI-1.bin");

        let expected = EDID {
            header: Header {
                vendor: ['D', 'E', 'L'],
                product: 41099,
                serial: 809851217,
                week: 15,
                year: 23,
                version: 1,
                revision: 3,
            },
            display: Display {
                video_input: 128,
                width: 53,
                height: 30,
                gamma: 120,
                features: 234,
            },
            chromaticity: (),
            established_timing: (),
            standard_timing: (),
            descriptors: vec![
                Descriptor::DetailedTiming(DetailedTiming {
                    pixel_clock: 148500,
                    horizontal_active_pixels: 1920,
                    horizontal_blanking_pixels: 280,
                    vertical_active_lines: 1080,
                    vertical_blanking_lines: 45,
                    horizontal_front_porch: 88,
                    horizontal_sync_width: 44,
                    vertical_front_porch: 4,
                    vertical_sync_width: 5,
                    horizontal_size: 531,
                    vertical_size: 299,
                    horizontal_border_pixels: 0,
                    vertical_border_pixels: 0,
                    features: 30,
                }),
                Descriptor::SerialNumber("67Y4J34A0EYQ".to_string()),
                Descriptor::ProductName("DELL S2440L".to_string()),
                Descriptor::RangeLimits,
            ],
            extensions: Some(CtaExtensions {
                extension_tag: 2,
                reserved: 3,
                native_dtd: NativeDTDs {
                    underscan: 1,
                    basic_audio: 1,
                    ycbcr444: 1,
                    ycbcr422: 1,
                    number_of_native_dtd: 1,
                },
                blocks: vec![
                    DataBlock::VideoBlock(VideoBlock {
                        header: DataBlockHeader {
                            type_tag: 2,
                            len: 12,
                        },
                        descriptors: vec![
                            ShortVideoDescriptor {
                                is_native: 1,
                                cea861_index: 16,
                            },
                            ShortVideoDescriptor {
                                is_native: 0,
                                cea861_index: 5,
                            },
                            ShortVideoDescriptor {
                                is_native: 0,
                                cea861_index: 4,
                            },
                            ShortVideoDescriptor {
                                is_native: 0,
                                cea861_index: 3,
                            },
                            ShortVideoDescriptor {
                                is_native: 0,
                                cea861_index: 2,
                            },
                            ShortVideoDescriptor {
                                is_native: 0,
                                cea861_index: 7,
                            },
                            ShortVideoDescriptor {
                                is_native: 0,
                                cea861_index: 22,
                            },
                            ShortVideoDescriptor {
                                is_native: 0,
                                cea861_index: 1,
                            },
                            ShortVideoDescriptor {
                                is_native: 0,
                                cea861_index: 20,
                            },
                            ShortVideoDescriptor {
                                is_native: 0,
                                cea861_index: 31,
                            },
                            ShortVideoDescriptor {
                                is_native: 0,
                                cea861_index: 18,
                            },
                            ShortVideoDescriptor {
                                is_native: 0,
                                cea861_index: 19,
                            },
                        ],
                    }),
                    DataBlock::AudioBlock(AudioBlock {
                        header: DataBlockHeader {
                            type_tag: 1,
                            len: 3,
                        },
                        descriptors: vec![ShortAudioDescriptor {
                            audio_format: 1,
                            number_of_channels: 2,
                            sampling_frequences: 7,
                            format_dependent_value: 7,
                            audio_format_extended_code: 0,
                        }],
                    }),
                    DataBlock::VendorSpecific(VendorSpecific {
                        header: DataBlockHeader {
                            type_tag: 3,
                            len: 5,
                        },
                        identifier: [3, 12, 0],
                        payload: vec![16, 0],
                    }),
                    DataBlock::SpeakerAllocation(SpeakerAllocation {
                        header: DataBlockHeader {
                            type_tag: 4,
                            len: 3,
                        },
                        speakers: 1,
                        reserved: [0, 0],
                    }),
                ],
                descriptors: vec![
                    DetailedTiming {
                        pixel_clock: 148500,
                        horizontal_active_pixels: 1920,
                        horizontal_blanking_pixels: 280,
                        vertical_active_lines: 1080,
                        vertical_blanking_lines: 45,
                        horizontal_front_porch: 88,
                        horizontal_sync_width: 44,
                        vertical_front_porch: 4,
                        vertical_sync_width: 5,
                        horizontal_size: 531,
                        vertical_size: 299,
                        horizontal_border_pixels: 0,
                        vertical_border_pixels: 0,
                        features: 30,
                    },
                    DetailedTiming {
                        pixel_clock: 74250,
                        horizontal_active_pixels: 1920,
                        horizontal_blanking_pixels: 280,
                        vertical_active_lines: 540,
                        vertical_blanking_lines: 22,
                        horizontal_front_porch: 88,
                        horizontal_sync_width: 44,
                        vertical_front_porch: 2,
                        vertical_sync_width: 5,
                        horizontal_size: 531,
                        vertical_size: 299,
                        horizontal_border_pixels: 0,
                        vertical_border_pixels: 0,
                        features: 158,
                    },
                    DetailedTiming {
                        pixel_clock: 74250,
                        horizontal_active_pixels: 1280,
                        horizontal_blanking_pixels: 370,
                        vertical_active_lines: 720,
                        vertical_blanking_lines: 30,
                        horizontal_front_porch: 110,
                        horizontal_sync_width: 40,
                        vertical_front_porch: 5,
                        vertical_sync_width: 5,
                        horizontal_size: 531,
                        vertical_size: 299,
                        horizontal_border_pixels: 0,
                        vertical_border_pixels: 0,
                        features: 30,
                    },
                    DetailedTiming {
                        pixel_clock: 27000,
                        horizontal_active_pixels: 720,
                        horizontal_blanking_pixels: 138,
                        vertical_active_lines: 480,
                        vertical_blanking_lines: 45,
                        horizontal_front_porch: 16,
                        horizontal_sync_width: 62,
                        vertical_front_porch: 9,
                        vertical_sync_width: 6,
                        horizontal_size: 531,
                        vertical_size: 299,
                        horizontal_border_pixels: 0,
                        vertical_border_pixels: 0,
                        features: 24,
                    },
                ],
            }),
        };

        test(d, &expected);
    }
}
