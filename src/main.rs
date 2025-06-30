use core::f32;
use std::fmt::Display;
use std::io::Read;

use clap::Parser;
use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone)]
enum ParseType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

#[derive(ValueEnum, Debug, Clone)]
enum ByteOrder {
    LittleEndian,
    BigEndian,
}

impl Display for ByteOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ByteOrder::LittleEndian => write!(f, "little-endian"),
            ByteOrder::BigEndian => write!(f, "big-endian"),
        }
    }
}

trait SizeOf {
    fn size_of(&self) -> i64;
}

impl SizeOf for ParseType {
    fn size_of(&self) -> i64 {
        let usize = match self {
            ParseType::U8 => std::mem::size_of::<u8>(),
            ParseType::U16 => std::mem::size_of::<u16>(),
            ParseType::U32 => std::mem::size_of::<u32>(),
            ParseType::U64 => std::mem::size_of::<u64>(),
            ParseType::I8 => std::mem::size_of::<i8>(),
            ParseType::I16 => std::mem::size_of::<i16>(),
            ParseType::I32 => std::mem::size_of::<i32>(),
            ParseType::I64 => std::mem::size_of::<i64>(),
            ParseType::F32 => std::mem::size_of::<f32>(),
            ParseType::F64 => std::mem::size_of::<f64>(),
        };
        usize as i64
    }
}

#[derive(Parser, Debug)]
struct Opt {
    #[clap(value_name = "TYPE")]
    parse_type: ParseType,

    #[clap(short, long, default_value_t = 0)]
    offset: u64,

    #[clap(short, long, default_value_t = i64::MAX)]
    number: i64,

    // Number of floats in each row, like 4 would print 4 floats per line
    #[clap(short, long, default_value_t = 1)]
    row_size: usize,

    #[clap(short, long, default_value_t = ByteOrder::LittleEndian)]
    byte_order: ByteOrder,

    file: String,
}

fn output<T: Display>(value: T, row_size: usize, current_row: &mut usize) {
    print!("{} ", value);
    *current_row += 1;
    if *current_row >= row_size {
        println!();
        *current_row = 0;
    }
}

fn main() {
    let args = Opt::parse();

    let file_size = match std::fs::metadata(&args.file) {
        Ok(meta) => {
            if meta.is_dir() {
                eprintln!("File is a directory: {}", args.file);
                std::process::exit(1);
            } else {
                // if windows, use the file size from the metadata
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::fs::MetadataExt;
                    meta.file_size()
                }
                // if unix, use the file size from the metadata
                #[cfg(not(target_os = "windows"))]
                {
                    use std::os::unix::fs::MetadataExt;
                    meta.size()
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    if args.offset >= file_size {
        eprintln!("Offset is out of range: {} >= {}", args.offset, file_size);
        std::process::exit(1);
    }

    let buffered_file_stream = std::fs::File::open(&args.file).unwrap();
    let mut file_stream = std::io::BufReader::new(buffered_file_stream);
    // If offset is different from 0, seek to the offset.
    if args.offset > 0 {
        match file_stream.seek_relative(args.offset as i64) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
    // While number is not 0, read the file and parse the data according to the type.
    let bytes_to_read = match args.number {
        i64::MAX => (file_size - args.offset) as i64,
        a => std::cmp::min(a * args.parse_type.size_of(), file_size as i64),
    };
    // Read the file until the end of the file or the number of bytes to read.
    let mut buffer = vec![0; 4096_usize];
    let mut previous_unread = 0;
    let mut bytes_read = 0;
    let mut current_row = 0;
    let row_count = args.row_size;
    while bytes_read < bytes_to_read {
        let bytes_to_read_now =
            std::cmp::min(buffer.len() as i64, bytes_to_read - bytes_read) as usize;
        match file_stream.read(&mut buffer[previous_unread..bytes_to_read_now]) {
            Ok(n) => {
                previous_unread = 0;
                if n == 0 {
                    break;
                }
                bytes_read += n as i64;
                let mut i = 0;
                while i < n {
                    let size = args.parse_type.size_of() as usize;
                    if i + size > n {
                        for j in 0..(n - i) {
                            buffer[j] = buffer[i + j];
                        }
                        previous_unread = n - i;
                        break;
                    }

                    match args.parse_type {
                        ParseType::U8 => {
                            let value = buffer[i] as u8;
                            output(value, row_count, &mut current_row);
                            i += 1;
                        }
                        ParseType::U16 => {
                            let value = match args.byte_order {
                                ByteOrder::LittleEndian => {
                                    u16::from_le_bytes([buffer[i], buffer[i + 1]])
                                }
                                ByteOrder::BigEndian => {
                                    u16::from_be_bytes([buffer[i], buffer[i + 1]])
                                }
                            };
                            output(value, row_count, &mut current_row);
                            i += 2;
                        }
                        ParseType::U32 => {
                            let value = match args.byte_order {
                                ByteOrder::LittleEndian => u32::from_le_bytes([
                                    buffer[i],
                                    buffer[i + 1],
                                    buffer[i + 2],
                                    buffer[i + 3],
                                ]),
                                ByteOrder::BigEndian => u32::from_be_bytes([
                                    buffer[i],
                                    buffer[i + 1],
                                    buffer[i + 2],
                                    buffer[i + 3],
                                ]),
                            };
                            output(value, row_count, &mut current_row);
                            i += 4;
                        }
                        ParseType::U64 => {
                            let value = match args.byte_order {
                                ByteOrder::LittleEndian => u64::from_le_bytes([
                                    buffer[i],
                                    buffer[i + 1],
                                    buffer[i + 2],
                                    buffer[i + 3],
                                    buffer[i + 4],
                                    buffer[i + 5],
                                    buffer[i + 6],
                                    buffer[i + 7],
                                ]),
                                ByteOrder::BigEndian => u64::from_be_bytes([
                                    buffer[i],
                                    buffer[i + 1],
                                    buffer[i + 2],
                                    buffer[i + 3],
                                    buffer[i + 4],
                                    buffer[i + 5],
                                    buffer[i + 6],
                                    buffer[i + 7],
                                ]),
                            };
                            output(value, row_count, &mut current_row);
                            i += 8;
                        }
                        ParseType::I8 => {
                            let value = buffer[i] as i8;
                            output(value, row_count, &mut current_row);
                            i += 1;
                        }
                        ParseType::I16 => {
                            let value = match args.byte_order {
                                ByteOrder::LittleEndian => {
                                    i16::from_le_bytes([buffer[i], buffer[i + 1]])
                                }
                                ByteOrder::BigEndian => {
                                    i16::from_be_bytes([buffer[i], buffer[i + 1]])
                                }
                            };
                            output(value, row_count, &mut current_row);
                        }
                        ParseType::I32 => {
                            let value = match args.byte_order {
                                ByteOrder::LittleEndian => i32::from_le_bytes([
                                    buffer[i],
                                    buffer[i + 1],
                                    buffer[i + 2],
                                    buffer[i + 3],
                                ]),
                                ByteOrder::BigEndian => i32::from_be_bytes([
                                    buffer[i],
                                    buffer[i + 1],
                                    buffer[i + 2],
                                    buffer[i + 3],
                                ]),
                            };
                            output(value, row_count, &mut current_row);
                            i += 4;
                        }
                        ParseType::I64 => {
                            let value = match args.byte_order {
                                ByteOrder::LittleEndian => i64::from_le_bytes([
                                    buffer[i],
                                    buffer[i + 1],
                                    buffer[i + 2],
                                    buffer[i + 3],
                                    buffer[i + 4],
                                    buffer[i + 5],
                                    buffer[i + 6],
                                    buffer[i + 7],
                                ]),
                                ByteOrder::BigEndian => i64::from_be_bytes([
                                    buffer[i],
                                    buffer[i + 1],
                                    buffer[i + 2],
                                    buffer[i + 3],
                                    buffer[i + 4],
                                    buffer[i + 5],
                                    buffer[i + 6],
                                    buffer[i + 7],
                                ]),
                            };
                            output(value, row_count, &mut current_row);
                            i += 8;
                        }
                        ParseType::F32 => {
                            let value = match args.byte_order {
                                ByteOrder::LittleEndian => f32::from_le_bytes([
                                    buffer[i],
                                    buffer[i + 1],
                                    buffer[i + 2],
                                    buffer[i + 3],
                                ]),
                                ByteOrder::BigEndian => f32::from_be_bytes([
                                    buffer[i],
                                    buffer[i + 1],
                                    buffer[i + 2],
                                    buffer[i + 3],
                                ]),
                            };
                            output(value, row_count, &mut current_row);
                            i += 4;
                        }
                        ParseType::F64 => {
                            let value = match args.byte_order {
                                ByteOrder::LittleEndian => f64::from_le_bytes([
                                    buffer[i],
                                    buffer[i + 1],
                                    buffer[i + 2],
                                    buffer[i + 3],
                                    buffer[i + 4],
                                    buffer[i + 5],
                                    buffer[i + 6],
                                    buffer[i + 7],
                                ]),
                                ByteOrder::BigEndian => f64::from_be_bytes([
                                    buffer[i],
                                    buffer[i + 1],
                                    buffer[i + 2],
                                    buffer[i + 3],
                                    buffer[i + 4],
                                    buffer[i + 5],
                                    buffer[i + 6],
                                    buffer[i + 7],
                                ]),
                            };
                            output(value, row_count, &mut current_row);
                            i += 8;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use super::*;

    #[test]
    fn test_size_of() {
        assert_eq!(ParseType::U8.size_of(), 1);
        assert_eq!(ParseType::U16.size_of(), 2);
        assert_eq!(ParseType::U32.size_of(), 4);
        assert_eq!(ParseType::U64.size_of(), 8);
        assert_eq!(ParseType::I8.size_of(), 1);
        assert_eq!(ParseType::I16.size_of(), 2);
        assert_eq!(ParseType::I32.size_of(), 4);
        assert_eq!(ParseType::I64.size_of(), 8);
        assert_eq!(ParseType::F32.size_of(), 4);
        assert_eq!(ParseType::F64.size_of(), 8);
    }

    #[test]
    fn write_sinus_cos_to_file() {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create("test_data.bin").unwrap();
        // Just write some fucking sinus and cosinus waves basically swinging from 1 to -1
        let steps = 10000;
        for i in 0..=steps {
            let t = i as f32 / steps as f32 * 2.0 * PI;
            let sin = t.sin();
            let bytes = sin.to_le_bytes();
            file.write_all(&bytes).unwrap();
            let cos = t.cos();
            let bytes = cos.to_le_bytes();
            file.write_all(&bytes).unwrap();
        }
    }
}
