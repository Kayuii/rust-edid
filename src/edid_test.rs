#[cfg(test)]
mod tests {
    use crate::edid::*;

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
        let d = include_bytes!("../testdata/card0-VGA-1.bin");

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
            chromaticity: (()),
            established_timing: (()),
            standard_timing: (()),
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
            extensions: None,
        };

        test(d, &expected);
    }

    #[test]
    fn test_card0_edp_1() {
        let d = include_bytes!("../testdata/card0-eDP-1.bin");

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
            chromaticity: (()),
            established_timing: (()),
            standard_timing: (()),
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
            extensions: None,
        };

        test(d, &expected);
    }
}