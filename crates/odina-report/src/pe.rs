// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! A PE (Windows executable) import hash — indicator 7 of the scoring model,
//! 12% of the weight, and until now permanently zero because nothing computed
//! it.
//!
//! "Imphash" is the industry name for a hash of the import table: the sequence
//! of `library!function` names a binary asks for at load time. It is a *shape*
//! of behaviour, not a fingerprint of bytes, so unrelated samples built from
//! the same builder share it. That is why it is weighted lower than a byte
//! hash and higher than a file name.
//!
//! What this module is honest about:
//!
//!   * it hashes the **sequence and names** of imports, lowercased, in the
//!     order the import directory lists them;
//!   * it does **not** reproduce Mandiant's imphash byte-for-byte (that
//!     algorithm is a specific library of quirks), and it says so instead of
//!     pretending the values are interchangeable;
//!   * any malformed file returns `None`. A parser that panics on a hostile
//!     input is a parser that hands the attacker a denial of service.

/// The result of reading a PE header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeInfo {
    /// 32 hex characters — MD5-shaped, and honestly named, because the field in
    /// `report-schema-v1.md` §2 is `string(32)`.
    pub imphash: String,
    /// Number of imported `library!function` pairs that went into the hash.
    pub import_count: usize,
}

const IMAGE_DIRECTORY_ENTRY_IMPORT: usize = 1;

fn u16_at(b: &[u8], off: usize) -> Option<u16> {
    b.get(off..off + 2)
        .map(|s| u16::from_le_bytes([s[0], s[1]]))
}

fn u32_at(b: &[u8], off: usize) -> Option<u32> {
    b.get(off..off + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

/// True when the first two bytes are `MZ`. Deliberately weak: a 2 KB read of a
/// `.docx` should not be parsed as a PE, but the stronger checks happen below.
pub fn looks_like_pe(head: &[u8]) -> bool {
    head.starts_with(b"MZ")
}

/// Read the import directory and hash it.
///
/// `head` must be the first bytes of the file — 4 KB is plenty for every real
/// PE header, because `e_lfanew` points at the NT headers within the first
/// kilobyte in practice. `read_at` supplies arbitrary offsets for the section
/// data that lives further into the file; returning `None` means the caller
/// cannot read that far, and the result is `None` rather than a partial hash.
pub fn parse<F>(head: &[u8], mut read_at: F) -> Option<PeInfo>
where
    F: FnMut(usize, usize) -> Option<Vec<u8>>,
{
    if !looks_like_pe(head) {
        return None;
    }

    let e_lfanew = u32_at(head, 0x3C)? as usize;
    let nt = read_at(e_lfanew, 0x18)?;
    if nt.get(0..4)? != b"PE\0\0" {
        return None;
    }

    let section_count = u16_at(&nt, 6)? as usize;
    let optional_size = u16_at(&nt, 20)? as usize;
    let optional = read_at(e_lfanew + 24, optional_size.max(2))?;

    let magic = u16_at(&optional, 0)?;
    // 0x10b = PE32, 0x20b = PE32+. The data directories sit after the
    // fixed-size part of the optional header, at a different offset per magic.
    let (dir_off, thunk_size) = match magic {
        0x10b => (96usize, 4usize),
        0x20b => (112usize, 8usize),
        _ => return None,
    };
    let import_rva = u32_at(&optional, dir_off + IMAGE_DIRECTORY_ENTRY_IMPORT * 8)? as usize;
    if import_rva == 0 {
        // A PE with no imports is legal (rare) and hashes to a stable value.
        return Some(PeInfo {
            imphash: md5_hex(b""),
            import_count: 0,
        });
    }

    // Section table: immediately after the optional header.
    let sections_base = e_lfanew + 24 + optional_size;
    let mut sections: Vec<(u32, u32, u32, u32)> = Vec::with_capacity(section_count);
    for i in 0..section_count {
        let s = read_at(sections_base + i * 40, 40)?;
        let vsize = u32_at(&s, 8)?;
        let va = u32_at(&s, 12)?;
        let raw_size = u32_at(&s, 16)?;
        let raw_ptr = u32_at(&s, 20)?;
        sections.push((va, vsize.max(raw_size), raw_ptr, raw_size));
    }

    let rva_to_off = |rva: usize| -> Option<usize> {
        for (va, vsize, raw_ptr, _) in &sections {
            let start = *va as usize;
            let end = start + *vsize as usize;
            if rva >= start && rva < end {
                return Some(*raw_ptr as usize + (rva - start));
            }
        }
        None
    };

    let mut descriptors_off = rva_to_off(import_rva)?;
    let mut buf = String::new();
    let mut import_count = 0usize;

    // 20 descriptors, terminated by an all-zero one.
    for _ in 0..512 {
        let desc = read_at(descriptors_off, 20)?;
        let original_thunk = u32_at(&desc, 0)? as usize;
        let name_rva = u32_at(&desc, 12)? as usize;
        let first_thunk = u32_at(&desc, 16)? as usize;
        if name_rva == 0 && first_thunk == 0 {
            break;
        }

        let lib_off = rva_to_off(name_rva)?;
        let lib = read_cstr(&mut read_at, lib_off)?;
        let thunk_rva = if original_thunk != 0 {
            original_thunk
        } else {
            first_thunk
        };

        let mut thunk_off = rva_to_off(thunk_rva)?;
        for _ in 0..4096 {
            let raw = read_at(thunk_off, thunk_size)?;
            let value = if thunk_size == 8 {
                u32_at(&raw, 0)? as u64 | ((u32_at(&raw, 4)? as u64) << 32)
            } else {
                u32_at(&raw, 0)? as u64
            };
            if value == 0 {
                break;
            }
            // The top bit means "import by ordinal": there is no function name
            // to hash, only a number, and a number is not a name.
            let by_ordinal = value >> (thunk_size * 8 - 1) == 1;
            if !by_ordinal {
                let hint_rva = value as usize;
                if let Some(off) = rva_to_off(hint_rva) {
                    if let Some(function) = read_cstr(&mut read_at, off + 2) {
                        buf.push_str(&lib.to_ascii_lowercase());
                        buf.push('!');
                        buf.push_str(&function.to_ascii_lowercase());
                        buf.push(',');
                        import_count += 1;
                    }
                }
            }
            thunk_off += thunk_size;
        }
        descriptors_off += 20;
    }

    Some(PeInfo {
        imphash: md5_hex(buf.as_bytes()),
        import_count,
    })
}

fn read_cstr<F>(read_at: &mut F, offset: usize) -> Option<String>
where
    F: FnMut(usize, usize) -> Option<Vec<u8>>,
{
    let mut out = Vec::new();
    let mut cursor = offset;
    loop {
        let chunk = read_at(cursor, 64)?;
        let end = chunk.iter().position(|b| *b == 0)?;
        out.extend_from_slice(&chunk[..end]);
        if end < chunk.len() {
            break;
        }
        cursor += chunk.len();
        if out.len() > 512 {
            return None; // a 512-byte symbol name is not a symbol name
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(&out).into_owned())
    }
}

/// MD5, written out in this file.
///
/// Why not a crate: `md5` would be the only dependency of a crate that
/// otherwise has none, and the algorithm is 60 lines of published constants.
/// The field is `string(32)` in the schema, which is MD5's length; the
/// function is used as an identifier, never as a security primitive. A
/// cryptographic hash here would make the value longer than the schema allows
/// without adding a single bit of usefulness — an import list is public.
pub fn md5_hex(input: &[u8]) -> String {
    let digest = md5(input);
    let mut out = String::with_capacity(32);
    const T: &[u8; 16] = b"0123456789abcdef";
    for b in digest {
        out.push(T[(b >> 4) as usize] as char);
        out.push(T[(b & 0x0f) as usize] as char);
    }
    out
}

fn md5(input: &[u8]) -> [u8; 16] {
    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    const K: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613,
        0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193,
        0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d,
        0x02441453, 0xd8a1e681, 0xe7d3fbc8, 0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122,
        0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
        0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665, 0xf4292244,
        0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb,
        0xeb86d391,
    ];

    let mut a0: u32 = 0x67452301;
    let mut b0: u32 = 0xefcdab89;
    let mut c0: u32 = 0x98badcfe;
    let mut d0: u32 = 0x10325476;

    let mut msg = input.to_vec();
    let bit_len = (input.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut m = [0u32; 16];
        for (i, word) in m.iter_mut().enumerate() {
            *word = u32::from_le_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }

        let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);
        for i in 0..64 {
            let (f, g) = match i {
                0..=15 => ((b & c) | (!b & d), i),
                16..=31 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                32..=47 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let tmp = d;
            d = c;
            c = b;
            b = b.wrapping_add(
                a.wrapping_add(f)
                    .wrapping_add(K[i])
                    .wrapping_add(m[g])
                    .rotate_left(S[i]),
            );
            a = tmp;
        }
        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut out = [0u8; 16];
    out[0..4].copy_from_slice(&a0.to_le_bytes());
    out[4..8].copy_from_slice(&b0.to_le_bytes());
    out[8..12].copy_from_slice(&c0.to_le_bytes());
    out[12..16].copy_from_slice(&d0.to_le_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three published MD5 vectors every implementation is expected to
    /// match. If these fail, the import hash is a random number.
    #[test]
    fn md5_matches_the_published_vectors() {
        assert_eq!(md5_hex(b""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(md5_hex(b"abc"), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(
            md5_hex(b"The quick brown fox jumps over the lazy dog"),
            "9e107d9d372bb6826bd81d3542a419d6"
        );
    }

    #[test]
    fn a_non_pe_is_rejected_without_reading_anything() {
        let mut reads = 0;
        let result = parse(b"this is not a pe", |_, _| {
            reads += 1;
            None
        });
        assert!(result.is_none());
        assert_eq!(reads, 0, "a non-PE must not trigger a single read");
    }

    #[test]
    fn a_truncated_header_is_rejected_not_panicked_on() {
        // Every prefix of a bogus PE header: none may panic.
        let mut data = b"MZ".to_vec();
        data.resize(0x40, 0);
        data[0x3c] = 0xff; // absurd e_lfanew
        for cut in 0..data.len() {
            let _ = parse(&data[..cut], |_, _| None);
        }
    }

    #[test]
    fn a_pe_head_alone_is_recognised() {
        assert!(looks_like_pe(b"MZ\x90\x00"));
        assert!(!looks_like_pe(b"PK\x03\x04"));
    }
}
