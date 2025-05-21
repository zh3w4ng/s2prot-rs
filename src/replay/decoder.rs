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
        // println!("Field name: {:?}", name);
        match protocol.type_infos.get(type_index) {
            Some(TypeInfo::Bool) => {
                // println!("Buffer: {}", self.buffer.data[self.buffer.byte_index]);
                self.buffer.expect_and_skip_byte(6);
                let name = name.to_string();
                let value = ParsedFieldType::Bool(self.buffer.read_bits(8) != 0);

                Ok((self.buffer, ParsedField { name, value }))
            }
            Some(TypeInfo::Optional { type_index }) => {
                // println!("Buffer: {}", self.buffer.data[self.buffer.byte_index]);
                self.buffer.expect_and_skip_byte(4);
                let exists = self.buffer.read_bits(8) != 0;
                let parsed_field = if exists {
                    let Ok((_, parsed_field)) = self.decode(name, *type_index, protocol) else {
                        panic!("Failed to decode TypeInfo::Optional");
                    };
                    parsed_field
                } else {
                    let name = name.to_string();
                    let value = ParsedFieldType::Optional(None);

                    ParsedField { name, value }
                };
                Ok((self.buffer, parsed_field))
            }
            Some(TypeInfo::Int {
                offset: _,
                length: _,
            }) => {
                // println!("Buffer: {}", self.buffer.data[self.buffer.byte_index]);
                self.buffer.expect_and_skip_byte(9);
                let name = name.to_string();
                let value = ParsedFieldType::Int(self.buffer.read_var_int());

                Ok((self.buffer, ParsedField { name, value }))
            }
            Some(TypeInfo::Blob {
                offset: _,
                length: _,
            }) => {
                // println!("Buffer: {}", self.buffer.data[self.buffer.byte_index]);
                self.buffer.expect_and_skip_byte(2);
                let name = name.to_string();
                let length = self.buffer.read_var_int() as usize;
                let bytes = self.buffer.read_aligned_bytes(length);
                let value = ParsedFieldType::Blob(bytes);

                Ok((self.buffer, ParsedField { name, value }))
            }
            Some(TypeInfo::Array {
                offset,
                length,
                type_index,
            }) => {
                // println!("Buffer: {}", self.buffer.data[self.buffer.byte_index]);
                self.buffer.expect_and_skip_byte(0);
                let name = name.to_string();
                let array_length = self.buffer.read_bits(*length) + *offset;
                let array = (0..array_length).map(|_| {
                    let Ok((_, parsed_field)) = self.decode("", *type_index, protocol) else {
                        panic!("Failed to decode TypeInfo::Array");
                    };
                    parsed_field.value
                });
                let value = ParsedFieldType::Array(array.collect());

                Ok((self.buffer, ParsedField { name, value }))
            }
            Some(TypeInfo::Struct { fields }) => {
                // println!("Buffer: {}", self.buffer.data[self.buffer.byte_index]);
                self.buffer.expect_and_skip_byte(5);
                let fields_length = self.buffer.read_var_int() as usize;
                // if fields_length != fields.len() {
                //     panic!(
                //         "Struct length mismatch: {} != {}",
                //         fields_length,
                //         fields.len()
                //     );
                // }
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
                let value = ParsedFieldType::Struct(parsed_fields);

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
                value: ParsedFieldType::Struct(vec![
                    ParsedField {
                        name: String::from("m_signature"),
                        value: ParsedFieldType::Blob(vec![
                            83, 116, 97, 114, 67, 114, 97, 102, 116, 32, 73, 73, 32, 114, 101, 112,
                            108, 97, 121, 27, 49, 49
                        ])
                    },
                    ParsedField {
                        name: String::from("m_version"),
                        value: ParsedFieldType::Struct(vec![
                            ParsedField {
                                name: String::from("m_flags"),
                                value: ParsedFieldType::Int(1)
                            },
                            ParsedField {
                                name: String::from("m_major"),
                                value: ParsedFieldType::Int(5)
                            },
                            ParsedField {
                                name: String::from("m_minor"),
                                value: ParsedFieldType::Int(0)
                            },
                            ParsedField {
                                name: String::from("m_revision"),
                                value: ParsedFieldType::Int(14)
                            },
                            ParsedField {
                                name: String::from("m_build"),
                                value: ParsedFieldType::Int(93272)
                            },
                            ParsedField {
                                name: String::from("m_baseBuild"),
                                value: ParsedFieldType::Int(93272)
                            }
                        ])
                    },
                    ParsedField {
                        name: String::from("m_type"),
                        value: ParsedFieldType::Int(2)
                    },
                    ParsedField {
                        name: String::from("m_elapsedGameLoops"),
                        value: ParsedFieldType::Int(11749)
                    },
                    ParsedField {
                        name: String::from("m_useScaledTime"),
                        value: ParsedFieldType::Bool(true)
                    },
                    ParsedField {
                        name: String::from("m_ngdpRootKey"),
                        value: ParsedFieldType::Struct(vec![ParsedField {
                            name: String::from("m_data"),
                            value: ParsedFieldType::Blob(vec![
                                82, 146, 10, 157, 137, 199, 246, 50, 53, 148, 93, 16, 243, 199, 60,
                                100
                            ])
                        }])
                    },
                    ParsedField {
                        name: String::from("m_dataBuildNum"),
                        value: ParsedFieldType::Int(93272)
                    },
                    ParsedField {
                        name: String::from("m_replayCompatibilityHash"),
                        value: ParsedFieldType::Struct(vec![ParsedField {
                            name: String::from("m_data"),
                            value: ParsedFieldType::Blob(vec![
                                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                            ])
                        }])
                    },
                    ParsedField {
                        name: String::from("m_ngdpRootKeyIsDevData"),
                        value: ParsedFieldType::Bool(false)
                    }
                ])
            }
        );
    }
}
