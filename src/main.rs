
// H and K is defined in https://nvlpubs.nist.gov/nistpubs/fips/nist.fips.180-4.pdf
const H: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

struct compress_h {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
    f: u32,
    g: u32,
    h: u32,
}

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

const TWO_POWERS: [u32; 32] = [
    1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072,
    262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216, 33554432, 67108864, 134217728,
    268435456, 536870912, 1073741824, 2147483648,
];

fn u64_to_bits(mut n: u64) -> [u8; 64] {
    let mut bits = [0 as u8; 64];
    for i in 0..64 {
        bits[63 - i] = (n & 1) as u8;
        n = n >> 1;
    }
    bits
}

fn convert_u8s_to_u32(v: &[u8]) -> u32 {
    let mut pow = 32;
    let mut ans: u32 = 0;
    for i in v {
        let mut x = *i as i32;

        for j in 0..8 {
            ans += TWO_POWERS[pow - 8 + j] * ((x & 1) as u32);
            x = x >> 1;
        }
        pow -= 8;
    }
    ans
}

fn convert_64_bits_to_u8(bits: [u8; 64]) -> [u8; 8] {
    let mut u8s: [u8; 8] = [0; 8];
    for i in 0..8 {
        let mut val: u8 = 0;
        for j in 0..8 {
            let ind = i * 8 + j;
            if bits[ind] == 1 {
                val += TWO_POWERS[7 - j] as u8;
            }
        }
        u8s[i] = val;
    }
    u8s
}

fn convert_u64_to_u8s(n: u64) -> [u8; 8] {
    convert_64_bits_to_u8(u64_to_bits(n))
}

fn pad_msg(v: &str) -> Vec<u8> {
    let mut msg_as_bytes: Vec<u8> = v.as_bytes().iter().map(|x| *x).collect();
    let original_msg_size: u64 = msg_as_bytes.len() as u64;

    msg_as_bytes.push(80); // to add bit 1 after the message: since we deal in bytes (u8) 1000_0000 = 80  
    let bytes_diff_from_64: u64 = 64 - 8 - (msg_as_bytes.len() % 64) as u64; // 64 - total bytes (512 bits); 8 - (reserving 8 bytes for original msg len)

    for _ in 0..bytes_diff_from_64 {
        msg_as_bytes.push(0);
    }

    for i in convert_u64_to_u8s(original_msg_size) {
        msg_as_bytes.push(i);
    }

    assert!(
        msg_as_bytes.len() % 64 == 0,
        "{}",
        format!(
            "Padding assert => missing {} bytes",
            msg_as_bytes.len() % 64
        )
    );

    print!("{:?} {}  ", msg_as_bytes, original_msg_size,);
    msg_as_bytes
}

fn prepare_msg(v: &[u8]) -> Vec<u32> {
    assert!(
        v.len() == 64,
        "Prepare_msg => expected len: 64, found {}",
        v.len()
    );
    let mut msg: Vec<u32> = Vec::with_capacity(64);
    let mut ind = 0;
    while ind < 64 {
        msg.push(convert_u8s_to_u32(&v[ind..(ind + 4)]));
        ind += 4;
    }
    assert!(
        msg.len() == 16,
        "Prepare_msg => expected len: 16, found {}",
        msg.len()
    );
    dbg!(&msg);
    msg
}

fn extend_to_64_words(mut v: Vec<u32>) -> Vec<u32> {
    assert!(v.len() == 16);
    assert!(v.capacity() == 64);

    for i in 16..64 {
        // s0 = (W[i-15] rightrotate 7) xor (W[i-15] rightrotate 18) xor (W[i-15] >> 3)
        let s0 = v[i - 15].rotate_right(7) ^ v[i - 15].rotate_right(18) ^ (v[i - 15] >> 3);
        // s1 = (W[i-2] rightrotate 17) xor (W[i-2] rightrotate 19) xor (W[i-2] >> 10)
        let s1 = v[i - 2].rotate_right(17) ^ v[i - 2].rotate_right(19) ^ (v[i - 2] >> 10);
        // W[i] = W[i-16] + s0 + W[i-7] + s1
        v.push(
            v[i - 16]
                .wrapping_add(s0)
                .wrapping_add(v[i - 7])
                .wrapping_add(s1),
        );
    }
    v
}

// for i = 0 to 63:
//     S1 = (e rightrotate 6) xor (e rightrotate 11) xor (e rightrotate 25)
//     ch = (e and f) xor ((not e) and g)
//     temp1 = h + S1 + ch + K[i] + W[i]

//     S0 = (a rightrotate 2) xor (a rightrotate 13) xor (a rightrotate 22)
//     maj = (a and b) xor (a and c) xor (b and c)
//     temp2 = S0 + maj

//     h = g
//     g = f
//     f = e
//     e = d + temp1
//     d = c
//     c = b
//     b = a
//     a = temp1 + temp2
fn compress_words(v: Vec<u32>, ch: &mut compress_h) {}

fn hash(v: &str) {
    let msg_as_bytes = pad_msg(v);
    let mut ind = 0;
    let mut ch = compress_h {
        a: H[0],
        b: H[1],
        c: H[2],
        d: H[3],
        e: H[4],
        f: H[5],
        g: H[6],
        h: H[7],
    };
    while ind < msg_as_bytes.len() {
        let words = prepare_msg(&msg_as_bytes[ind..(ind + 64)]);
        let words = extend_to_64_words(words);
        assert!(words.len() == 64);
        compress_words(words, &mut ch);
        ind += 64;
    }
}

fn main() {
    let input = "abdasdsc";
    hash(input);
}
