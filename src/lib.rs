// H and K is defined in https://nvlpubs.nist.gov/nistpubs/fips/nist.fips.180-4.pdf
const H: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

struct CompressH {
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

fn pad_msg(v: &str) -> Vec<u8> {
    let mut msg_as_bytes: Vec<u8> = v.as_bytes().to_vec();
    let original_msg_size: u64 = msg_as_bytes.len() as u64;

    msg_as_bytes.push(128); // to add bit 1 after the message: since we deal in bytes (u8) 1000_0000 = 128  
    let bytes_diff_from_64: u64 = 64 - 8 - (msg_as_bytes.len() % 64) as u64; // 64 - total bytes (512 bits); 8 - (reserving 8 bytes for original msg len)

    for _ in 0..bytes_diff_from_64 {
        msg_as_bytes.push(0);
    }

    msg_as_bytes.extend_from_slice(&(original_msg_size * 8).to_be_bytes()); // original msg size in bits not bytes

    assert!(
        msg_as_bytes.len() % 64 == 0,
        "{}",
        format!(
            "Padding assert => missing {} bytes",
            msg_as_bytes.len() % 64
        )
    );

    // println!("{:?} in bits {}  ", msg_as_bytes, original_msg_size * 8,);
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
        // msg.push(convert_u8s_to_u32(&v[ind..(ind + 4)]));
        msg.push(u32::from_be_bytes([
            v[ind],
            v[ind + 1],
            v[ind + 2],
            v[ind + 3],
        ]));
        ind += 4;
    }
    assert!(
        msg.len() == 16,
        "Prepare_msg => expected len: 16, found {}",
        msg.len()
    );
    // dbg!(&msg);
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

fn compress_words(v: Vec<u32>, ch: &mut CompressH) {
    for i in 0..64 {
        let s1 = ch.e.rotate_right(6) ^ ch.e.rotate_right(11) ^ ch.e.rotate_right(25);
        let ch1 = (ch.e & ch.f) ^ ((!ch.e) & ch.g);
        let temp1 =
            ch.h.wrapping_add(s1)
                .wrapping_add(ch1)
                .wrapping_add(K[i])
                .wrapping_add(v[i]);
        let s0 = ch.a.rotate_right(2) ^ ch.a.rotate_right(13) ^ ch.a.rotate_right(22);
        let maj = (ch.a & ch.b) ^ (ch.a & ch.c) ^ (ch.b & ch.c);
        let temp2 = s0.wrapping_add(maj);
        ch.h = ch.g;
        ch.g = ch.f;
        ch.f = ch.e;
        ch.e = ch.d.wrapping_add(temp1);
        ch.d = ch.c;
        ch.c = ch.b;
        ch.b = ch.a;
        ch.a = temp1.wrapping_add(temp2);
    }
}

pub fn hash(v: &str) -> String {
    let msg_as_bytes = pad_msg(v);
    assert!(msg_as_bytes.len() % 64 == 0);
    let mut ind = 0;

    let mut h = H;
    while ind < msg_as_bytes.len() {
        let mut ch = CompressH {
            a: h[0],
            b: h[1],
            c: h[2],
            d: h[3],
            e: h[4],
            f: h[5],
            g: h[6],
            h: h[7],
        };
        let cha_bfr = ch.a;
        let words = prepare_msg(&msg_as_bytes[ind..(ind + 64)]);
        let words = extend_to_64_words(words);
        assert!(words.len() == 64);
        compress_words(words, &mut ch);
        let cha_aft = ch.a;
        assert!(cha_bfr != cha_aft);
        ind += 64;
        h[0] = h[0].wrapping_add(ch.a);
        h[1] = h[1].wrapping_add(ch.b);
        h[2] = h[2].wrapping_add(ch.c);
        h[3] = h[3].wrapping_add(ch.d);
        h[4] = h[4].wrapping_add(ch.e);
        h[5] = h[5].wrapping_add(ch.f);
        h[6] = h[6].wrapping_add(ch.g);
        h[7] = h[7].wrapping_add(ch.h);
    }
    let mut ans = String::new();
    ans.extend(h.iter().map(|x| format!("{:08x}", x)));
    // println!("{ans}");
    ans
}
