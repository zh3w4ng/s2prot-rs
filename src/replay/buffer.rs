use std::cmp::Ordering;

#[derive(Debug, Clone, Copy)]
pub struct BitPackedBuff<'a> {
    pub data: &'a [u8],
    pub byte_index: usize,
    cache: u8,
    big_endian: bool,
    bits_in_cache: usize,
}

impl<'a> BitPackedBuff<'a> {
    const LOWEST_BITS_MASK: [u8; 9] = [0x00, 0x01, 0x03, 0x07, 0x0f, 0x1f, 0x3f, 0x7f, 0xff];

    fn new(data: &'a [u8], big_endian: bool) -> Self {
        BitPackedBuff {
            data,
            big_endian,
            cache: 0,
            byte_index: 0,
            bits_in_cache: 0,
        }
    }

    pub fn new_little_endian(data: &'a [u8]) -> Self {
        Self::new(data, false)
    }

    pub fn new_big_endian(data: &'a [u8]) -> Self {
        Self::new(data, true)
    }
    pub fn display(&self) {
        println!("Buff value: {:?}", self.data);
    }

    pub fn read_bits(&mut self, n: usize) -> isize {
        if self.big_endian {
            self.read_bits_big(n)
        } else {
            self.read_bits_little(n)
        }
    }
    pub fn read_bit_array(&mut self, bits: usize) -> Vec<u8> {
        let mut res = self.read_unaligned_bytes(bits / 8);
        if bits % 8 != 0 {
            res.push(self.read_bits(bits % 8) as u8);
        }
        res
    }
    pub fn expect_and_skip_byte(&mut self, expected: u8) {
        if self.data[self.byte_index] != expected {
            panic!(
                "Expected byte: {}, but found: {}",
                expected, self.data[self.byte_index]
            );
        } else {
            self.byte_index += 1;
        }
    }

    fn read_bits_little(&mut self, mut n: usize) -> isize {
        let mut bits_in_value: usize = 0;
        let mut value: isize = 0;

        loop {
            self.init_cache();

            match n.cmp(&self.bits_in_cache) {
                Ordering::Greater => {
                    value |= (self.cache as isize) << bits_in_value;
                    n -= self.bits_in_cache;
                    bits_in_value += self.bits_in_cache;
                    self.byte_align();
                }
                Ordering::Less => {
                    value |= ((self.cache & BitPackedBuff::LOWEST_BITS_MASK[n]) as isize)
                        << bits_in_value;
                    self.bits_in_cache -= n;
                    self.cache >>= n;
                    break;
                }
                Ordering::Equal => {
                    value |= (self.cache as isize) << bits_in_value;
                    self.byte_align();
                    break;
                }
            }
        }
        value
    }

    fn read_bits_big(&mut self, mut n: usize) -> isize {
        let mut value: isize = 0;
        loop {
            self.init_cache();
            match n.cmp(&self.bits_in_cache) {
                Ordering::Greater => {
                    value = (value << (self.bits_in_cache)) | (self.cache as isize);
                    n -= self.bits_in_cache;
                    self.byte_align();
                }
                Ordering::Less => {
                    value =
                        (value << n) | ((self.cache & BitPackedBuff::LOWEST_BITS_MASK[n]) as isize);
                    self.bits_in_cache -= n;
                    self.cache >>= n;
                    break;
                }
                Ordering::Equal => {
                    value = (value << n) | (self.cache as isize);
                    self.byte_align();
                    break;
                }
            }
        }
        value
    }

    fn init_cache(&mut self) {
        if self.bits_in_cache == 0 {
            self.cache = self.data[self.byte_index];
            self.byte_index += 1;
            self.bits_in_cache = 8;
        }
    }

    pub fn byte_align(&mut self) {
        self.bits_in_cache = 0;
    }

    pub fn read_aligned_bytes(&mut self, n: usize) -> Vec<u8> {
        let mut vec = vec![0; n];
        self.byte_align();
        for el in vec.iter_mut() {
            *el = self.data[self.byte_index];
            self.byte_index += 1;
        }
        vec
    }

    pub fn read_unaligned_bytes(&mut self, n: usize) -> Vec<u8> {
        let mut vec = vec![0; n];
        for el in vec.iter_mut() {
            *el = self.read_bits(8) as u8;
        }
        vec
    }

    pub fn skip_bytes(&mut self, n: usize) -> &Self {
        self.byte_index += n;

        self
    }

    /// readVarInt reads a variable-length int value.
    /// Format: read from input by 8 bits.
    ///     * Highest bit tells if have to read more bytes,
    ///     * Lowest bit of the firt byte (first 8 bits) is not data but tells if the number is negative.
    pub fn read_var_int(&mut self) -> isize {
        let mut value: isize = 0;
        let mut shift = 0;
        let mut byte: isize;
        loop {
            byte = self.read_bits(8);
            value |= (byte & 0x7f) << shift;
            if (byte & 0x80) == 0 {
                break;
            } else {
                shift += 7;
            }
        }
        if value & 0x01 > 0 {
            -(value >> 1)
        } else {
            value >> 1
        }
    }

    pub fn read_int(&mut self, length: usize, offset: isize) -> isize {
        offset + self.read_bits(length)
    }

    pub fn done(&self) -> bool {
        self.bits_in_cache == 0 && self.byte_index >= self.data.len()
    }
}

#[cfg(test)]
mod tests {
    fn fixture() -> &'static [u8] {
        b"\x00\xae+\x03\x00\xa4\x00\x00\x00\xe7\x03\x00\x00\xbf\x0b\x00\x00\x01traP\xe7\x03\x00\x00\xbf\x0b\x00\x00\x02traP\xe7\x03\x00\x00\xf4\x01\x00\x00\x01nmuH\xe7\x03\x00\x00\xf4\x01\x00\x00\x02nmuH\xe7\x03\x00\x00\xb9\x0b\x00\x00\x01rreT\xe7\x03\x00\x00\xb9\x0b\x00\x00\x02torP\xe7\x03\x00\x00\x1f\x0c\x00\x00\x0100BA\xe7\x03\x00\x00\x1f\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\xc3\x0b\x00\x00\x010   \xe7\x03\x00\x00\xc3\x0b\x00\x00\x020   \xe7\x03\x00\x00$\x0c\x00\x00\x0100BA\xe7\x03\x00\x00$\x0c\x00\x00\x0200BA\xe7\x03\x00\x00`\x0c\x00\x00\x0100BA\xe7\x03\x00\x00`\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\xe8\x07\x00\x00\x012T\x00\x00\xe7\x03\x00\x00\xe8\x07\x00\x00\x021T\x00\x00\xe7\x03\x00\x00\xc4\x0b\x00\x00\x0122  \xe7\x03\x00\x00\xc4\x0b\x00\x00\x0222  \xe7\x03\x00\x00\x8d\x13\x00\x00\x010   \xe7\x03\x00\x00\x8d\x13\x00\x00\x020   \xe7\x03\x00\x00&\x0c\x00\x00\x0100BA\xe7\x03\x00\x00&\x0c\x00\x00\x0200BA\xe7\x03\x00\x00%\x0c\x00\x00\x0100BA\xe7\x03\x00\x00%\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\x82\x0c\x00\x00\x0100BA\xe7\x03\x00\x00\x82\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\xba\x0b\x00\x00\x0110ct\xe7\x03\x00\x00\xba\x0b\x00\x00\x0220ct\xe7\x03\x00\x00\xd8\x07\x00\x00\x012T\x00\x00\xe7\x03\x00\x00\xd8\x07\x00\x00\x021T\x00\x00\xe7\x03\x00\x00~\x0c\x00\x00\x0100BA\xe7\x03\x00\x00~\x0c\x00\x00\x0200BA\xe7\x03\x00\x00a\x0c\x00\x00\x0100BA\xe7\x03\x00\x00a\x0c\x00\x00\x0200BA\xe7\x03\x00\x00g\x0c\x00\x00\x0100BA\xe7\x03\x00\x00g\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\xd4\x07\x00\x00\x012T\x00\x00\xe7\x03\x00\x00\xd4\x07\x00\x00\x021T\x00\x00\xe7\x03\x00\x00\xcc\x0b\x00\x00\x01on\x00\x00\xe7\x03\x00\x00\xcc\x0b\x00\x00\x02on\x00\x00\xe7\x03\x00\x00C\x0c\x00\x00\x0100BA\xe7\x03\x00\x00C\x0c\x00\x00\x0200BA\xe7\x03\x00\x00P\x14\x00\x00\x01on\x00\x00\xe7\x03\x00\x00P\x14\x00\x00\x02on\x00\x00\xe7\x03\x00\x00\xd3\x07\x00\x00\x012T\x00\x00\xe7\x03\x00\x00\xd3\x07\x00\x00\x021T\x00\x00\xe7\x03\x00\x00\x7f\x0c\x00\x00\x0100BA\xe7\x03\x00\x00\x7f\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\x84\x0c\x00\x00\x0100BA\xe7\x03\x00\x00\x84\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\xc8\x0b\x00\x00\x010   \xe7\x03\x00\x00\xc8\x0b\x00\x00\x020   \xe7\x03\x00\x00@\x0c\x00\x00\x0100BA\xe7\x03\x00\x00@\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\xa0\x0f\x00\x00\x10HMoN\xe7\x03\x00\x00\xbc\x0b\x00\x00\x01ideM\xe7\x03\x00\x00\xbc\x0b\x00\x00\x02ideM\xe7\x03\x00\x00\xdb\x07\x00\x00\x012T\x00\x00\xe7\x03\x00\x00\xdb\x07\x00\x00\x021T\x00\x00\xe7\x03\x00\x00\xb4\x14\x00\x00\x010   \xe7\x03\x00\x00\xb4\x14\x00\x00\x020   \xe7\x03\x00\x00\x8a\x13\x00\x00\x010   \xe7\x03\x00\x00\x8a\x13\x00\x00\x020   \xe7\x03\x00\x00\xd0\x07\x00\x00\x102t\x00\x00\xe7\x03\x00\x00\x86\x0c\x00\x00\x0100BA\xe7\x03\x00\x00\x86\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\xa5\x0f\x00\x00\x01on\x00\x00\xe7\x03\x00\x00\xa5\x0f\x00\x00\x02on\x00\x00\xe7\x03\x00\x00\xb8\x0b\x00\x00\x10rsaF\xe7\x03\x00\x00\x85\x0c\x00\x00\x0100BA\xe7\x03\x00\x00\x85\x0c\x00\x00\x0200BA\xe7\x03\x00\x00b\x0c\x00\x00\x0100BA\xe7\x03\x00\x00b\x0c\x00\x00\x0200BA\xe7\x03\x00\x00A\x0c\x00\x00\x0100BA\xe7\x03\x00\x00A\x0c\x00\x00\x0200BA\xe7\x03\x00\x00^\x0c\x00\x00\x0100BA\xe7\x03\x00\x00^\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\xbe\x0b\x00\x00\x1001\x00\x00\xe7\x03\x00\x00\xe9\x03\x00\x00\x10sey\x00\xe7\x03\x00\x00\xc1\x0b\x00\x00\x10virP\xe7\x03\x00\x00G\x0c\x00\x00\x0100BA\xe7\x03\x00\x00G\x0c\x00\x00\x0200BA\xe7\x03\x00\x00#\x0c\x00\x00\x0100BA\xe7\x03\x00\x00#\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\xcb\x0b\x00\x00\x01on\x00\x00\xe7\x03\x00\x00\xcb\x0b\x00\x00\x02on\x00\x00\xe7\x03\x00\x00_\x0c\x00\x00\x0100BA\xe7\x03\x00\x00_\x0c\x00\x00\x0200BA\xe7\x03\x00\x00d\x0c\x00\x00\x0100BA\xe7\x03\x00\x00d\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\xe2\x07\x00\x00\x012T\x00\x00\xe7\x03\x00\x00\xe2\x07\x00\x00\x021T\x00\x00\xe7\x03\x00\x00 \x0c\x00\x00\x0100BA\xe7\x03\x00\x00 \x0c\x00\x00\x0200BA\xe7\x03\x00\x00\xd7\x07\x00\x00\x012T\x00\x00\xe7\x03\x00\x00\xd7\x07\x00\x00\x021T\x00\x00\xe7\x03\x00\x00\xbb\x0b\x00\x00\x01001 \xe7\x03\x00\x00\xbb\x0b\x00\x00\x02001 \xe7\x03\x00\x00\xc5\x0b\x00\x00\x01\x00\x00\x00\x00\xe7\x03\x00\x00\xc5\x0b\x00\x00\x02\x00\x00\x00\x00\xe7\x03\x00\x00\xd6\x07\x00\x00\x012T\x00\x00\xe7\x03\x00\x00\xd6\x07\x00\x00\x021T\x00\x00\xe7\x03\x00\x00f\x0c\x00\x00\x0100BA\xe7\x03\x00\x00f\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\xd2\x07\x00\x00\x012T\x00\x00\xe7\x03\x00\x00\xd2\x07\x00\x00\x021T\x00\x00\xe7\x03\x00\x00\xc6\x0b\x00\x00\x011   \xe7\x03\x00\x00\xc6\x0b\x00\x00\x021   \xe7\x03\x00\x00e\x0c\x00\x00\x0100BA\xe7\x03\x00\x00e\x0c\x00\x00\x0200BA\xe7\x03\x00\x00B\x0c\x00\x00\x0100BA\xe7\x03\x00\x00B\x0c\x00\x00\x0200BA\xe7\x03\x00\x00!\x0c\x00\x00\x0100BA\xe7\x03\x00\x00!\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\xc7\x0b\x00\x00\x100\x00\x00\x00\xe7\x03\x00\x00>\x0c\x00\x00\x0100BA\xe7\x03\x00\x00>\x0c\x00\x00\x0200BA\xe7\x03\x00\x00'\x0c\x00\x00\x0100BA\xe7\x03\x00\x00'\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\x83\x0c\x00\x00\x0100BA\xe7\x03\x00\x00\x83\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\x8c\x13\x00\x00\x010   \xe7\x03\x00\x00\x8c\x13\x00\x00\x020   \xe7\x03\x00\x00?\x0c\x00\x00\x0100BA\xe7\x03\x00\x00?\x0c\x00\x00\x0200BA\xe7\x03\x00\x00D\x0c\x00\x00\x0100BA\xe7\x03\x00\x00D\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\x8b\x13\x00\x00\x010   \xe7\x03\x00\x00\x8b\x13\x00\x00\x020   \xe7\x03\x00\x00\x80\x0c\x00\x00\x0100BA\xe7\x03\x00\x00\x80\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\xd5\x07\x00\x00\x012T\x00\x00\xe7\x03\x00\x00\xd5\x07\x00\x00\x021T\x00\x00\xe7\x03\x00\x00\xe8\x03\x00\x00\x10tlfD\xe7\x03\x00\x00\xc0\x0b\x00\x00\x01sbO\x00\xe7\x03\x00\x00\xc0\x0b\x00\x00\x02sbO\x00\xe7\x03\x00\x00\xc2\x0b\x00\x00\x10sey\x00\xe7\x03\x00\x00\x89\x13\x00\x00\x010   \xe7\x03\x00\x00\x89\x13\x00\x00\x020   \xe7\x03\x00\x00F\x0c\x00\x00\x0100BA\xe7\x03\x00\x00F\x0c\x00\x00\x0200BA\xae+\x03\x00\x01\x00\x00\x00\x011000\xae+\x03\x00\x01\x00\x00\x00\x021000\xe7\x03\x00\x00\xc9\x0b\x00\x00\x010   \xe7\x03\x00\x00\xc9\x0b\x00\x00\x020   \xe7\x03\x00\x00E\x0c\x00\x00\x0100BA\xe7\x03\x00\x00E\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\"\x0c\x00\x00\x0100BA\xe7\x03\x00\x00\"\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\xa1\x0f\x00\x00\x01on\x00\x00\xe7\x03\x00\x00\xa1\x0f\x00\x00\x02on\x00\x00\xe7\x03\x00\x00\x81\x0c\x00\x00\x0100BA\xe7\x03\x00\x00\x81\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\x1e\x0c\x00\x00\x0100BA\xe7\x03\x00\x00\x1e\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\x87\x0c\x00\x00\x0100BA\xe7\x03\x00\x00\x87\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\x88\x13\x00\x00\x010   \xe7\x03\x00\x00\x88\x13\x00\x00\x020   \xe7\x03\x00\x00c\x0c\x00\x00\x0100BA\xe7\x03\x00\x00c\x0c\x00\x00\x0200BA\xe7\x03\x00\x00\xd1\x07\x00\x00\x101v1\x00\xe7\x03\x00\x00\xec\x13\x00\x00\x010   \xe7\x03\x00\x00\xec\x13\x00\x00\x020"
    }

    #[cfg(test)]
    mod big_edidian {
        use super::super::*;
        use byteorder::{BigEndian, ByteOrder};
        use tests::fixture;

        #[test]
        fn it_reads_bits() {
            let mut buff = BitPackedBuff::new_big_endian(fixture());
            assert_eq!("0", format!("{:x}", buff.read_bits(8)));
            assert_eq!("ae2b0300", format!("{:x}", buff.read_bits(32)));
            assert_eq!("a4000000", format!("{:x}", buff.read_bits(32)));
        }
        #[test]
        fn it_is_equivalant_to_bigedian_read_32() {
            let data: [u8; 4] = [0xae, 0x2b, 0x03, 0x00];
            assert_eq!("ae2b0300", format!("{:x}", BigEndian::read_u32(&data)));
        }

        #[test]
        fn it_reads_aligned_bytes() {
            let data: [u8; 6] = [5, 18, 0, 2, 44, 83];
            let mut buff = BitPackedBuff::new_big_endian(&data);
            assert_eq!(0, buff.byte_index);
            let bytes = buff.read_aligned_bytes(4);
            assert_eq!(4, buff.byte_index);
            assert_eq!(vec![5, 18, 0, 2], bytes);
        }

        #[test]
        fn it_reads_int() {
            let data: [u8; 2] = [1, 18];
            let mut buff = BitPackedBuff::new_big_endian(&data);
            let res = buff.read_int(16, 1);
            assert_eq!(275, res);
        }
    }

    #[cfg(test)]
    mod little_endian {
        use super::super::*;
        use byteorder::{ByteOrder, LittleEndian};
        use tests::fixture;

        #[test]
        fn it_reads_bits() {
            let mut buff = BitPackedBuff::new_little_endian(fixture());
            assert_eq!("0", format!("{:x}", buff.read_bits(8)));
            assert_eq!("32bae", format!("{:x}", buff.read_bits(32)));
            assert_eq!("a4", format!("{:x}", buff.read_bits(32)));
        }
        #[test]
        fn it_is_equivalant_to_littleedian_read_32() {
            let data: [u8; 4] = [0xae, 0x2b, 0x03, 0x00];
            assert_eq!("32bae", format!("{:x}", LittleEndian::read_u32(&data)));
        }

        #[test]
        fn it_reads_var_int() {
            let data: [u8; 2] = [44, 83];
            let mut buff = BitPackedBuff::new_little_endian(&data);
            assert_eq!(22, buff.read_var_int());
        }

        #[test]
        fn it_reads_var_int_that_is_larger_than_256() {
            let data: [u8; 3] = [176, 177, 11];
            let mut buff = BitPackedBuff::new_little_endian(&data);
            assert_eq!(93272, buff.read_var_int());
        }

        #[test]
        fn it_expects_and_skips_bytes() {
            let data: [u8; 6] = [5, 18, 0, 2, 44, 83];
            let mut buff = BitPackedBuff::new_little_endian(&data);
            assert_eq!(0, buff.byte_index);
            buff.expect_and_skip_byte(5);
            assert_eq!(1, buff.byte_index);
        }

        #[test]
        fn it_skips_bytes() {
            let data: [u8; 6] = [5, 18, 0, 2, 44, 83];
            let mut buff = BitPackedBuff::new_little_endian(&data);
            assert_eq!(0, buff.byte_index);
            buff.skip_bytes(4);
            assert_eq!(4, buff.byte_index);
        }
    }
}
