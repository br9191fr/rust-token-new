#[cfg(test)]
mod tests {
    use data_encoding::{HEXLOWER, BASE64};
    use error_chain::{error_chain};
    use std::fs::File;
    use std::io::Read;
    use tokio::fs::File;
    use tokio::io::AsyncReadExt; // for read_to_end()

    error_chain! {
    foreign_links {
        Io(std::io::Error);
        ParseInt(::std::num::ParseIntError);
        }
    }
    fn read_uptime() -> Result<u64> {
        let mut uptime = String::new();
        File::open("/proc/xxxuptime")?.read_to_string(&mut uptime)?;

        Ok(uptime
            .split('.')
            .next()
            .ok_or("Cannot parse uptime data")?
            .parse()?)
    }

    #[test]
    async fn test_readasync() {
        let mut file = File::open("foo.txt").await?;

        let mut contents = vec![];
        file.read_to_end(&mut contents).await?;

        println!("len = {}", contents.len());
    }

    #[test]
    fn test_base64_encoding() {
        assert_eq!(BASE64.encode(b"Hello world"), "SGVsbG8gd29ybGQ=");
    }

    #[test]
    fn test_base64_decoding() {
        //
        //assert_eq!(BASE64.decode(b"SGVsbG8gd29ybGQK").unwrap(), b"Hello world");
        assert_eq!(BASE64.decode(b"SGVsbG8gd29ybGQ=").unwrap(), b"Hello world");
        assert_eq!(BASE64.decode(b"SGVsbA==byB3b3JsZA==").unwrap(), b"Hello world");
    }

    #[test]
    fn test_base64_decoding2() {
        let mut buffer = [0u8; 64];
        let input = b"SGVsbA==byB3b3JsZA==";
        let len = BASE64.decode_len(input.len()).unwrap();
        println!("len is {}", len);
        assert_eq!(len, 15);
        let output = &mut buffer[0..BASE64.decode_len(input.len()).unwrap()];
        let len = BASE64.decode_mut(input, output).unwrap();
        assert_eq!(&output[0..len], b"Hello world");
        //assert_eq!(BASE64.decode(b"SGVsbG8gd29ybGQ=").unwrap(), b"Hello world");
    }

    #[test]
    fn test_uptime() {
        match read_uptime() {
            Ok(uptime) => println!("uptime: {} seconds", uptime),
            Err(err) => eprintln!("error: {}", err),
        };
    }
}




