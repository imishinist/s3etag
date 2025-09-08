# s3etag

Calculate S3 ETag for multipart uploads.

## Installation

```bash
cargo install --path . --features cli
```

## Usage

### Library

```rust
use s3etag;

let digest = s3etag::compute(b"hello");
println!("{:x}", digest); // 62109206880d38a4010a98e11243924a-1
```

### CLI

```bash
# Calculate ETag with default 8MB chunk size
s3etag file.txt

# Calculate ETag with custom chunk size (in MB)
s3etag -c 16 file.txt

# Verify against expected ETag
s3etag -e "expected-etag-here" file.txt
```

## License

MIT
