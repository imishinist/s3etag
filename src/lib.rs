//! A library for computing chunked MD5 digests.
//!
//! ## Example
//!
//! ```
//! let digest = s3etag::compute(b"hello");
//! assert_eq!(format!("{:x}", digest), "62109206880d38a4010a98e11243924a-1");
//! ```
//!

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Digest {
    hash: [u8; 16],
    parts: usize,
}

impl Digest {
    #[inline]
    pub fn hash(&self) -> &[u8; 16] {
        &self.hash
    }

    #[inline]
    pub fn parts(&self) -> usize {
        self.parts
    }
}

impl core::convert::From<Digest> for [u8; 16] {
    #[inline]
    fn from(digest: Digest) -> Self {
        digest.hash
    }
}

macro_rules! implement {
    ($kind:ident, $format:expr) => {
        impl core::fmt::$kind for Digest {
            fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                for value in &self.hash {
                    write!(formatter, $format, value)?;
                }
                write!(formatter, "-{}", self.parts)?;
                Ok(())
            }
        }
    };
}

implement!(LowerHex, "{:02x}");
implement!(UpperHex, "{:02X}");

impl core::fmt::Display for Digest {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::LowerHex::fmt(self, f)
    }
}

impl core::ops::Deref for Digest {
    type Target = [u8; 16];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.hash
    }
}

impl core::ops::DerefMut for Digest {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.hash
    }
}

#[derive(Clone)]
pub struct Context {
    combined_hashes: Vec<u8>,
    current_chunk: Vec<u8>,
    chunk_size: usize,
    chunk_count: usize,
    total_bytes: u64,
}

impl Context {
    #[inline]
    pub fn new() -> Self {
        Self::with_chunk_size(8 * 1024 * 1024)
    }

    #[inline]
    pub fn with_chunk_size(chunk_size: usize) -> Self {
        Self {
            combined_hashes: Vec::new(),
            current_chunk: Vec::with_capacity(chunk_size),
            chunk_size,
            chunk_count: 0,
            total_bytes: 0,
        }
    }

    #[inline]
    pub fn with_capacity(chunk_size: usize, total_size: u64) -> Self {
        let estimated_chunks = total_size.div_ceil(chunk_size as u64) as usize;
        Self {
            combined_hashes: Vec::with_capacity(estimated_chunks * 16),
            current_chunk: Vec::with_capacity(chunk_size),
            chunk_size,
            chunk_count: 0,
            total_bytes: 0,
        }
    }

    pub fn consume<T: AsRef<[u8]>>(&mut self, data: T) {
        let data = data.as_ref();
        self.total_bytes += data.len() as u64;
        let mut remaining = data;

        while !remaining.is_empty() {
            let space_left = self.chunk_size - self.current_chunk.len();
            let to_take = remaining.len().min(space_left);

            self.current_chunk.extend_from_slice(&remaining[..to_take]);
            remaining = &remaining[to_take..];

            if self.current_chunk.len() == self.chunk_size {
                let hash = md5::compute(&self.current_chunk);
                self.combined_hashes.extend_from_slice(&hash.0);
                self.current_chunk.clear();
                self.chunk_count += 1;
            }
        }
    }

    pub fn finalize(mut self) -> Digest {
        if !self.current_chunk.is_empty() {
            let hash = md5::compute(&self.current_chunk);
            self.combined_hashes.extend_from_slice(&hash.0);
            self.chunk_count += 1;
        }

        let final_hash = md5::compute(&self.combined_hashes);
        Digest {
            hash: final_hash.0,
            parts: self.chunk_count,
        }
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
}

impl Default for Context {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl core::convert::From<Context> for Digest {
    #[inline]
    fn from(ctx: Context) -> Self {
        ctx.finalize()
    }
}

impl std::io::Write for Context {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.consume(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Compute the digest of data with default chunk size (8 MiB).
#[inline]
pub fn compute<T: AsRef<[u8]>>(data: T) -> Digest {
    let mut ctx = Context::new();
    ctx.consume(data);
    ctx.finalize()
}

/// Compute the digest of data with specified chunk size in bytes.
#[inline]
pub fn compute_with_chunk_size<T: AsRef<[u8]>>(data: T, chunk_size: usize) -> Digest {
    let mut ctx = Context::with_chunk_size(chunk_size);
    ctx.consume(data);
    ctx.finalize()
}

#[cfg(test)]
mod tests {
    use super::Context;

    #[test]
    fn compute() {
        let large = "a".repeat(8 * 1024 * 1024 + 1);
        let inputs = ["hello", "hello\n", &large];
        let outputs = [
            "62109206880d38a4010a98e11243924a-1",
            "6a6d8d4533507d490ab007dfe8314ab7-1",
            "b62778a5dbf858d29ac84f718e3a8374-2",
        ];
        for (input, &output) in inputs.iter().zip(outputs.iter()) {
            let digest = super::compute(input);
            assert_eq!(format!("{digest:x}"), output);

            let mut context = Context::new();
            context.consume(input);
            let digest = context.finalize();
            assert_eq!(format!("{digest:x}"), output);
        }

        let inputs = vec![("a".repeat(8 * 1024 * 1024 + 1), 2)];
        let outputs = vec!["2b26d4c146cf1500e532eed66eba4a36-5"];
        for ((input, chunk_size), &output) in inputs.iter().zip(outputs.iter()) {
            let digest = super::compute_with_chunk_size(input, *chunk_size * 1024 * 1024);
            assert_eq!(format!("{digest:x}"), output);

            let mut context = Context::with_chunk_size(*chunk_size * 1024 * 1024);
            context.consume(input);
            let digest = context.finalize();
            assert_eq!(format!("{digest:x}"), output);
        }
    }
}
