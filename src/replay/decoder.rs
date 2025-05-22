extern crate nom;

use nom::IResult;

use super::buffer::BitPackedBuff;
use super::types::*;
use crate::protocol::types::Protocol;
use crate::protocol::types::TypeInfo;

pub struct Decoder<'a> {
    buffer: BitPackedBuff<'a>,
}

impl<'a> Decoder<'a> {
    pub fn new(buffer: BitPackedBuff<'a>) -> Self {
        Self { buffer }
    }

    pub fn decode(
        &mut self,
        name: &'a str,
        type_index: usize,
        protocol: &'a Protocol,
    ) -> IResult<BitPackedBuff<'a>, ParsedField> {
        println!("Field name: {:?}", name);
        match protocol.type_infos.get(type_index) {
            Some(TypeInfo::Bool) => {
                println!("Buffer: {}", self.buffer.data[self.buffer.byte_index]);
                self.buffer.expect_and_skip_byte(6);
                let name = name.to_string();
                let value = Some(ParsedFieldType::Bool(self.buffer.read_bits(8) != 0));

                Ok((self.buffer, ParsedField { name, value }))
            }
            Some(TypeInfo::Optional { type_index }) => {
                println!("Buffer: {}", self.buffer.data[self.buffer.byte_index]);
                self.buffer.expect_and_skip_byte(4);
                let exists = self.buffer.read_bits(8) != 0;
                let parsed_field = if exists {
                    let Ok((_, parsed_field)) = self.decode(name, *type_index, protocol) else {
                        panic!("Failed to decode TypeInfo::Optional");
                    };
                    parsed_field
                } else {
                    let name = name.to_string();
                    let value = None;

                    ParsedField { name, value }
                };
                Ok((self.buffer, parsed_field))
            }
            Some(TypeInfo::Int {
                offset: _,
                length: _,
            }) => {
                println!("Buffer: {}", self.buffer.data[self.buffer.byte_index]);
                self.buffer.expect_and_skip_byte(9);
                let name = name.to_string();
                let value = Some(ParsedFieldType::Int(self.buffer.read_var_int()));
                println!("Parsed value: {:?}", value);

                Ok((self.buffer, ParsedField { name, value }))
            }
            Some(TypeInfo::Blob {
                offset: _,
                length: _,
            }) => {
                println!("Buffer: {}", self.buffer.data[self.buffer.byte_index]);
                self.buffer.expect_and_skip_byte(2);
                let name = name.to_string();
                let length = self.buffer.read_var_int() as usize;
                let bytes = self.buffer.read_aligned_bytes(length);
                let chars = String::from_utf8_lossy(&bytes).into_owned();
                let value = Some(ParsedFieldType::Blob(chars));

                Ok((self.buffer, ParsedField { name, value }))
            }
            Some(TypeInfo::FourCC) => {
                println!("Buffer: {}", self.buffer.data[self.buffer.byte_index]);
                self.buffer.expect_and_skip_byte(7);
                let name = name.to_string();
                let value = Some(ParsedFieldType::FourCC(self.buffer.read_aligned_bytes(4)));

                Ok((self.buffer, ParsedField { name, value }))
            }
            Some(TypeInfo::Array {
                offset: _,
                length: _,
                type_index,
            }) => {
                println!("Buffer: {}", self.buffer.data[self.buffer.byte_index]);
                self.buffer.expect_and_skip_byte(0);
                let name = name.to_string();
                let array_length = self.buffer.read_var_int() as usize;
                println!("Array length: {}", array_length);
                let array = (0..array_length).map(|_| {
                    let Ok((_, parsed_field)) = self.decode("", *type_index, protocol) else {
                        panic!("Failed to decode TypeInfo::Array");
                    };
                    parsed_field.value.unwrap()
                });
                let value = Some(ParsedFieldType::Array(array.collect()));

                Ok((self.buffer, ParsedField { name, value }))
            }
            Some(TypeInfo::Struct { fields }) => {
                println!("Buffer: {}", self.buffer.data[self.buffer.byte_index]);
                self.buffer.expect_and_skip_byte(5);
                let fields_length = self.buffer.read_var_int() as usize;
                let mut parsed_fields: Vec<ParsedField> = Vec::with_capacity(fields_length);
                let mut i = 0;
                while i < fields_length {
                    let tag = self.buffer.read_var_int();
                    while tag > fields.get(i).unwrap().tag {
                        i += 1;
                    }
                    let field = fields.get(i).unwrap();

                    match self.decode(&field.name, field.type_index, protocol) {
                        Ok((_, parsed_field)) => {
                            parsed_fields.push(parsed_field);
                        }
                        _ => panic!("Failed to decode TypeInfo::Struct::Field: {}", field.name),
                    }
                    i += 1;
                }
                let name = name.to_string();
                let value = Some(ParsedFieldType::Struct(parsed_fields));

                Ok((self.buffer, ParsedField { name, value }))
            }
            others => {
                panic!("Unknown TypeInfo: {}", others.unwrap());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load_protocol_version;

    #[test]
    fn it_parse_user_data_with_no_error() {
        let protocol = load_protocol_version("93272");
        let index: usize = protocol.replay_header_type_index.unwrap();
        let input: &[u8] = &[
            5, 18, 0, 2, 44, 83, 116, 97, 114, 67, 114, 97, 102, 116, 32, 73, 73, 32, 114, 101,
            112, 108, 97, 121, 27, 49, 49, 2, 5, 12, 0, 9, 2, 2, 9, 10, 4, 9, 0, 6, 9, 28, 8, 9,
            176, 177, 11, 10, 9, 176, 177, 11, 4, 9, 4, 6, 9, 202, 183, 1, 8, 6, 1, 10, 5, 2, 2, 2,
            32, 82, 146, 10, 157, 137, 199, 246, 50, 53, 148, 93, 16, 243, 199, 60, 100, 12, 9,
            176, 177, 11, 14, 5, 2, 2, 2, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16,
            6, 0,
        ];
        let bit_packed_buffer = BitPackedBuff::new_big_endian(input);
        let mut decoder = Decoder::new(bit_packed_buffer);
        let (_, user_data) = decoder.decode("UserData", index, &protocol).unwrap();
        assert_eq!(
            user_data,
            ParsedField {
                name: String::from("UserData"),
                value: Some(ParsedFieldType::Struct(vec![
                    ParsedField {
                        name: String::from("m_signature"),
                        value: Some(ParsedFieldType::Blob(String::from(
                            "StarCraft II replay\u{1b}11"
                        )))
                    },
                    ParsedField {
                        name: String::from("m_version"),
                        value: Some(ParsedFieldType::Struct(vec![
                            ParsedField {
                                name: String::from("m_flags"),
                                value: Some(ParsedFieldType::Int(1))
                            },
                            ParsedField {
                                name: String::from("m_major"),
                                value: Some(ParsedFieldType::Int(5))
                            },
                            ParsedField {
                                name: String::from("m_minor"),
                                value: Some(ParsedFieldType::Int(0))
                            },
                            ParsedField {
                                name: String::from("m_revision"),
                                value: Some(ParsedFieldType::Int(14))
                            },
                            ParsedField {
                                name: String::from("m_build"),
                                value: Some(ParsedFieldType::Int(93272))
                            },
                            ParsedField {
                                name: String::from("m_baseBuild"),
                                value: Some(ParsedFieldType::Int(93272))
                            }
                        ]))
                    },
                    ParsedField {
                        name: String::from("m_type"),
                        value: Some(ParsedFieldType::Int(2))
                    },
                    ParsedField {
                        name: String::from("m_elapsedGameLoops"),
                        value: Some(ParsedFieldType::Int(11749))
                    },
                    ParsedField {
                        name: String::from("m_useScaledTime"),
                        value: Some(ParsedFieldType::Bool(true))
                    },
                    ParsedField {
                        name: String::from("m_ngdpRootKey"),
                        value: Some(ParsedFieldType::Struct(vec![ParsedField {
                            name: String::from("m_data"),
                            value: Some(ParsedFieldType::Blob(String::from(
                                "R�\n����25�]\u{10}��<d"
                            )))
                        }]))
                    },
                    ParsedField {
                        name: String::from("m_dataBuildNum"),
                        value: Some(ParsedFieldType::Int(93272))
                    },
                    ParsedField {
                        name: String::from("m_replayCompatibilityHash"),
                        value: Some(ParsedFieldType::Struct(vec![ParsedField {
                            name: String::from("m_data"),
                            value: Some(ParsedFieldType::Blob(String::from(
                                "\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"
                            )))
                        }]))
                    },
                    ParsedField {
                        name: String::from("m_ngdpRootKeyIsDevData"),
                        value: Some(ParsedFieldType::Bool(false))
                    }
                ]))
            }
        );
    }

    #[test]
    fn it_parse_details_data_with_no_error() {
        let protocol = load_protocol_version("93272");
        let index: usize = protocol.game_details_type_index.unwrap();
        let input: &[u8] = &[
            5, 36, 0, 4, 1, 0, 4, 5, 22, 0, 2, 12, 103, 117, 109, 105, 104, 111, 2, 5, 8, 0, 9, 4,
            2, 7, 0, 0, 83, 50, 4, 9, 2, 8, 9, 162, 161, 218, 3, 4, 2, 12, 84, 101, 114, 114, 97,
            110, 6, 5, 8, 0, 9, 254, 3, 2, 9, 232, 2, 4, 9, 40, 6, 9, 60, 8, 9, 4, 10, 9, 2, 12, 9,
            200, 1, 14, 9, 0, 16, 9, 2, 18, 4, 1, 9, 22, 20, 2, 0, 5, 22, 0, 2, 54, 38, 108, 116,
            59, 109, 108, 101, 109, 38, 103, 116, 59, 60, 115, 112, 47, 62, 76, 105, 113, 117, 105,
            100, 67, 108, 101, 109, 2, 5, 8, 0, 9, 4, 2, 7, 0, 0, 83, 50, 4, 9, 2, 8, 9, 144, 196,
            255, 2, 4, 2, 14, 80, 114, 111, 116, 111, 115, 115, 6, 5, 8, 0, 9, 254, 3, 2, 9, 0, 4,
            9, 132, 1, 6, 9, 254, 3, 8, 9, 4, 10, 9, 0, 12, 9, 200, 1, 14, 9, 0, 16, 9, 4, 18, 4,
            1, 9, 28, 20, 2, 0, 2, 2, 30, 65, 98, 121, 115, 115, 97, 108, 32, 82, 101, 101, 102,
            32, 76, 69, 4, 2, 0, 6, 5, 2, 0, 2, 22, 77, 105, 110, 105, 109, 97, 112, 46, 116, 103,
            97, 8, 6, 1, 10, 9, 254, 157, 221, 142, 254, 148, 162, 219, 3, 12, 9, 128, 160, 163,
            156, 140, 2, 14, 2, 0, 16, 2, 0, 18, 2, 0, 20, 4, 1, 0, 18, 2, 80, 115, 50, 109, 97, 0,
            0, 69, 85, 109, 228, 21, 3, 186, 204, 208, 86, 86, 54, 11, 111, 2, 125, 184, 129, 105,
            250, 25, 137, 187, 99, 87, 177, 178, 21, 162, 84, 121, 57, 245, 251, 2, 80, 115, 50,
            109, 97, 0, 0, 69, 85, 66, 28, 138, 160, 243, 97, 155, 101, 45, 35, 162, 115, 93, 254,
            232, 18, 171, 100, 66, 40, 35, 94, 122, 121, 126, 222, 207, 232, 182, 125, 163, 14, 2,
            80, 115, 50, 109, 97, 0, 0, 69, 85, 102, 9, 56, 50, 18, 132, 83, 239, 255, 187, 120,
            124, 128, 183, 211, 238, 193, 173, 129, 189, 229, 92, 131, 201, 48, 222, 167, 156, 78,
            80, 90, 4, 2, 80, 115, 50, 109, 97, 0, 0, 69, 85, 217, 45, 252, 72, 196, 132, 197, 145,
            84, 39, 11, 146, 74, 215, 213, 116, 132, 242, 171, 154, 71, 98, 28, 122, 177, 100, 49,
            191, 102, 197, 59, 64, 2, 80, 115, 50, 109, 97, 0, 0, 69, 85, 156, 86, 111, 161, 186,
            173, 97, 26, 155, 197, 202, 130, 113, 199, 36, 67, 19, 77, 76, 82, 152, 61, 50, 203,
            202, 17, 5, 71, 68, 205, 253, 140, 2, 80, 115, 50, 109, 97, 0, 0, 69, 85, 108, 5, 82,
            114, 71, 68, 217, 130, 96, 89, 250, 73, 252, 30, 210, 38, 245, 17, 59, 110, 59, 108,
            44, 128, 99, 224, 13, 188, 188, 99, 126, 31, 2, 80, 115, 50, 109, 97, 0, 0, 69, 85, 85,
            46, 44, 76, 212, 1, 179, 206, 35, 154, 90, 56, 125, 202, 242, 178, 19, 67, 80, 68, 241,
            116, 210, 155, 28, 225, 198, 208, 149, 224, 220, 139, 2, 80, 115, 50, 109, 97, 0, 0,
            69, 85, 127, 65, 65, 26, 165, 151, 244, 180, 100, 64, 212, 42, 86, 51, 72, 191, 83,
            130, 45, 42, 104, 17, 47, 1, 4, 249, 184, 145, 246, 240, 90, 225, 2, 80, 115, 50, 109,
            97, 0, 0, 69, 85, 43, 164, 101, 117, 16, 239, 31, 122, 17, 172, 172, 115, 175, 50, 109,
            71, 20, 232, 62, 94, 114, 97, 249, 177, 241, 241, 185, 167, 203, 255, 42, 250, 22, 6,
            0, 24, 9, 8, 26, 9, 6, 28, 4, 0, 30, 9, 0, 32, 4, 1, 6, 0, 34, 6, 0,
        ];
        let bit_packed_buffer = BitPackedBuff::new_big_endian(input);
        let mut decoder = Decoder::new(bit_packed_buffer);
        let (_, details_data) = decoder.decode("DetailsData", index, &protocol).unwrap();
        assert_eq!(
            details_data,
            ParsedField {
                name: String::from("DetailsData"),
                value: Some(ParsedFieldType::Struct(vec![
                    ParsedField { name: String::from("m_playerList"),
                        value: Some(ParsedFieldType::Array(vec![
                            ParsedFieldType::Struct(vec![
                                ParsedField { name: String::from("m_name"), value: Some(ParsedFieldType::Blob(String::from("gumiho"))) },
                                ParsedField { name: String::from("m_toon"), value: Some(ParsedFieldType::Struct(vec![
                                    ParsedField { name: String::from("m_region"), value: Some(ParsedFieldType::Int(2)) },
                                    ParsedField { name: String::from("m_programId"), value: Some(ParsedFieldType::FourCC(vec![0, 0, 83, 50])) },
                                    ParsedField { name: String::from("m_realm"), value: Some(ParsedFieldType::Int(1)) },
                                    ParsedField { name: String::from("m_id"), value: Some(ParsedFieldType::Int(3885137)) }
                                ])) },
                                ParsedField { name: String::from("m_race"), value: Some(ParsedFieldType::Blob(String::from("Terran"))) },
                                ParsedField { name: String::from("m_color"), value: Some(ParsedFieldType::Struct(vec![
                                    ParsedField { name: String::from("m_a"), value: Some(ParsedFieldType::Int(255)) },
                                    ParsedField { name: String::from("m_r"), value: Some(ParsedFieldType::Int(180)) },
                                    ParsedField { name: String::from("m_g"), value: Some(ParsedFieldType::Int(20)) },
                                    ParsedField { name: String::from("m_b"), value: Some(ParsedFieldType::Int(30)) }
                                ])) },
                                ParsedField { name: String::from("m_control"), value: Some(ParsedFieldType::Int(2)) },
                                ParsedField { name: String::from("m_teamId"), value: Some(ParsedFieldType::Int(1)) },
                                ParsedField { name: String::from("m_handicap"), value: Some(ParsedFieldType::Int(100)) },
                                ParsedField { name: String::from("m_observe"), value: Some(ParsedFieldType::Int(0)) },
                                ParsedField { name: String::from("m_result"), value: Some(ParsedFieldType::Int(1)) },
                                ParsedField { name: String::from("m_workingSetSlotId"), value: Some(ParsedFieldType::Int(11)) },
                                ParsedField { name: String::from("m_hero"), value: Some(ParsedFieldType::Blob(String::from(""))) }
                            ]),
                            ParsedFieldType::Struct(vec![
                                ParsedField { name: String::from("m_name"), value: Some(ParsedFieldType::Blob(String::from("&lt;mlem&gt;<sp/>LiquidClem"))) },
                                ParsedField { name: String::from("m_toon"), value: Some(ParsedFieldType::Struct(vec![
                                    ParsedField { name: String::from("m_region"), value: Some(ParsedFieldType::Int(2)) },
                                    ParsedField { name: String::from("m_programId"), value: Some(ParsedFieldType::FourCC(vec![0, 0, 83, 50])) },
                                    ParsedField { name: String::from("m_realm"), value: Some(ParsedFieldType::Int(1)) },
                                    ParsedField { name: String::from("m_id"), value: Some(ParsedFieldType::Int(3141896)) }
                                ])) },
                                ParsedField { name: String::from("m_race"), value: Some(ParsedFieldType::Blob(String::from("Protoss"))) },
                                ParsedField { name: String::from("m_color"), value: Some(ParsedFieldType::Struct(vec![
                                    ParsedField { name: String::from("m_a"), value: Some(ParsedFieldType::Int(255)) },
                                    ParsedField { name: String::from("m_r"), value: Some(ParsedFieldType::Int(0)) },
                                    ParsedField { name: String::from("m_g"), value: Some(ParsedFieldType::Int(66)) },
                                    ParsedField { name: String::from("m_b"), value: Some(ParsedFieldType::Int(255)) }
                                ])) },
                                ParsedField { name: String::from("m_control"), value: Some(ParsedFieldType::Int(2)) },
                                ParsedField { name: String::from("m_teamId"), value: Some(ParsedFieldType::Int(0)) },
                                ParsedField { name: String::from("m_handicap"), value: Some(ParsedFieldType::Int(100)) },
                                ParsedField { name: String::from("m_observe"), value: Some(ParsedFieldType::Int(0)) },
                                ParsedField { name: String::from("m_result"), value: Some(ParsedFieldType::Int(2)) },
                                ParsedField { name: String::from("m_workingSetSlotId"), value: Some(ParsedFieldType::Int(14)) },
                                ParsedField { name: String::from("m_hero"), value: Some(ParsedFieldType::Blob(String::from(""))) }])
                            ])) },
                            ParsedField { name: String::from("m_title"), value: Some(ParsedFieldType::Blob(String::from("Abyssal Reef LE"))) },
                            ParsedField { name: String::from("m_difficulty"), value: Some(ParsedFieldType::Blob(String::from(""))) },
                            ParsedField { name: String::from("m_thumbnail"), value: Some(ParsedFieldType::Struct(vec![
                                ParsedField { name: String::from("m_file"), value: Some(ParsedFieldType::Blob(String::from("Minimap.tga"))) }
                            ])) },
                            ParsedField { name: String::from("m_isBlizzardMap"), value: Some(ParsedFieldType::Bool(true)) },
                            ParsedField { name: String::from("m_timeUTC"), value: Some(ParsedFieldType::Int(133775741252511615)) },
                            ParsedField { name: String::from("m_timeLocalOffset"), value: Some(ParsedFieldType::Int(36000000000)) },
                            ParsedField { name: String::from("m_description"), value: Some(ParsedFieldType::Blob(String::from(""))) },
                            ParsedField { name: String::from("m_imageFilePath"), value: Some(ParsedFieldType::Blob(String::from(""))) },
                            ParsedField { name: String::from("m_mapFileName"), value: Some(ParsedFieldType::Blob(String::from(""))) },
                            ParsedField { name: String::from("m_cacheHandles"), value: Some(ParsedFieldType::Array(vec![
                                ParsedFieldType::Blob(String::from("s2ma\0\0EUm�\u{15}\u{3}���VV6\u{b}o\u{2}}��i�\u{19}��cW��\u{15}�Ty9��")),
                                ParsedFieldType::Blob(String::from("s2ma\0\0EUB\u{1c}���a�e-#�s]��\u{12}�dB(#^zy~���}�\u{e}")),
                                ParsedFieldType::Blob(String::from("s2ma\0\0EUf\t82\u{12}�S���x|���������\\��0\u{7a7}�NPZ\u{4}")),
                                ParsedFieldType::Blob(String::from("s2ma\0\0EU�-�HĄőT'\u{b}�J��t��Gb\u{1c}z�d1�f�;@")),
                                ParsedFieldType::Blob(String::from("s2ma\0\0EU�Vo���a\u{1a}��ʂq�$C\u{13}MLR�=2��\u{11}\u{5}GD���")),
                                ParsedFieldType::Blob(String::from("s2ma\0\0EUl\u{5}RrGDق`Y�I�\u{1e}�&�\u{11};n;l,�c�\r��c~\u{1f}")),
                                ParsedFieldType::Blob(String::from("s2ma\0\0EUU.,L�\u{1}��#�Z8}��\u{13}CPD�tқ\u{1c}��Е�܋")),
                                ParsedFieldType::Blob(String::from("s2ma\0\0EU\u{7f}AA\u{1a}����d@�*V3H�S�-*h\u{11}/\u{1}\u{4}�����Z�")),
                                ParsedFieldType::Blob(String::from("s2ma\0\0EU+�eu\u{10}�\u{1f}z\u{11}��s�2mG\u{14}�>^ra������*�"))
                            ])) },
                            ParsedField { name: String::from("m_miniSave"), value: Some(ParsedFieldType::Bool(false)) },
                            ParsedField { name: String::from("m_gameSpeed"), value: Some(ParsedFieldType::Int(4)) },
                            ParsedField { name: String::from("m_defaultDifficulty"), value: Some(ParsedFieldType::Int(3)) },
                            ParsedField { name: String::from("m_modPaths"), value: None },
                            ParsedField { name: String::from("m_campaignIndex"), value: Some(ParsedFieldType::Int(0)) },
                            ParsedField { name: String::from("m_restartAsTransitionMap"), value: Some(ParsedFieldType::Bool(false)) },
                            ParsedField { name: String::from("m_disableRecoverGame"), value: Some(ParsedFieldType::Bool(false)) }])) }
                );
    }
}
