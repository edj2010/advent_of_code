const S: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9,
    14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15,
    21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

const K: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

const INITIAL_STATE: [u32; 4] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476];

fn collapse_slice(slice: &[u8]) -> u32 {
    ((slice[3] as u32) << 24)
        | ((slice[2] as u32) << 16)
        | ((slice[1] as u32) << 8)
        | (slice[0] as u32)
}

fn collapse_state(state: &[u32]) -> u128 {
    ((state[0].swap_bytes() as u128) << 96)
        | ((state[1].swap_bytes() as u128) << 64)
        | ((state[2].swap_bytes() as u128) << 32)
        | (state[3].swap_bytes() as u128)
}

struct ClusterIter<'a> {
    bytes: &'a [u8],
    next_cluster_offset: usize,
}

impl<'a> ClusterIter<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        ClusterIter {
            bytes,
            next_cluster_offset: 0,
        }
    }
}

impl<'a> Iterator for ClusterIter<'a> {
    type Item = [u32; 16];

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_cluster_offset >= self.bytes.len() {
            None
        } else {
            let mut cluster: [u32; 16] = [0; 16];
            for word_offset in
                (self.next_cluster_offset..(self.next_cluster_offset + 64)).step_by(4)
            {
                cluster[(word_offset - self.next_cluster_offset) / 4] =
                    collapse_slice(&self.bytes[word_offset..(word_offset + 4)]);
            }
            self.next_cluster_offset += 64;
            Some(cluster)
        }
    }
}

fn apply_cluster(cluster: [u32; 16], state: [u32; 4]) -> [u32; 4] {
    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut f: u32;
    let mut g: u32;
    for i in 0..64 {
        // println!("{} {} {} {}", a, b, c, d);

        if i < 16 {
            f = (b & c) | ((!b) & d);
            g = i;
        } else if i < 32 {
            f = (d & b) | ((!d) & c);
            g = (5 * i + 1) % 16
        } else if i < 48 {
            f = (b ^ c) ^ d;
            g = (3 * i + 5) % 16;
        } else {
            f = c ^ (b | (!d));
            g = (7 * i) % 16;
        }

        // if i == 41 {
        //     print!("{} {} ", cluster[g as usize], f);
        // }
        f = f
            .wrapping_add(a)
            .wrapping_add(K[i as usize])
            .wrapping_add(cluster[g as usize]);
        a = d;
        d = c;
        c = b;
        b = b.wrapping_add(f.rotate_left(S[i as usize]));

        // if i == 41 {
        //     println!("{} {} {}", f, f.rotate_left(S[i as usize]), b);
        // }
    }
    [
        state[0].wrapping_add(a),
        state[1].wrapping_add(b),
        state[2].wrapping_add(c),
        state[3].wrapping_add(d),
    ]
}

fn prep_byte_array<I: IntoIterator<Item = u8>>(byte_iter: I) -> Vec<u8> {
    let mut bytes: Vec<u8> = byte_iter.into_iter().collect();
    let message_length = (bytes.len() as u64).wrapping_mul(8);
    bytes.push(0b10000000);
    while bytes.len() % 64 != 56 {
        bytes.push(0);
    }
    for offset in (0..=56).step_by(8) {
        bytes.push(((message_length >> offset) & 0xff) as u8);
    }
    bytes
}

pub fn hash_bytes<I: IntoIterator<Item = u8>>(byte_iter: I) -> u128 {
    let bytes = prep_byte_array(byte_iter);
    let mut state = INITIAL_STATE;
    for cluster in ClusterIter::new(&bytes) {
        state = apply_cluster(cluster, state);
    }
    collapse_state(&state)
}

pub fn hash_string(s: &str) -> u128 {
    hash_bytes(s.bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shl_test() {
        let start: u32 = 0b01010111011010101010010001110111;
        assert_eq!(start.rotate_left(1), 0b10101110110101010100100011101110);
        assert_eq!(start.rotate_left(2), 0b01011101101010101001000111011101);
        assert_eq!(start.rotate_left(7), 0b10110101010100100011101110101011);
    }

    #[test]
    fn cluster_iter_empty_string() {
        let clusters: Vec<[u32; 16]> = ClusterIter::new(&prep_byte_array([])).collect();
        assert_eq!(clusters.len(), 1);
        assert_eq!(
            clusters[0],
            [128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn cluster_iter_a() {
        let clusters: Vec<[u32; 16]> = ClusterIter::new(&prep_byte_array([b'a'])).collect();
        assert_eq!(clusters.len(), 1);
        assert_eq!(
            clusters[0],
            [32865, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0]
        );
    }

    #[test]
    fn cluster_iter_quick_brown_fox() {
        let clusters: Vec<[u32; 16]> = ClusterIter::new(&prep_byte_array(
            "The quick brown fox jumps over the lazy dog".bytes(),
        ))
        .collect();
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0][14], 344);
    }

    #[test]
    fn empty_string() {
        assert_eq!(hash_string(""), 0xd41d8cd98f00b204e9800998ecf8427e);
    }

    #[test]
    fn quick_brown_fox() {
        assert_eq!(
            hash_string("The quick brown fox jumps over the lazy dog"),
            0x9e107d9d372bb6826bd81d3542a419d6
        );
    }

    #[test]
    fn quick_brown_fox_sentence() {
        assert_eq!(
            hash_string("The quick brown fox jumps over the lazy dog."),
            0xe4d909c290d0fb1ca068ffaddf22cbd0
        );
    }
}
