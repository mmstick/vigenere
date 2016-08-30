use rustc_serialize::hex::{ToHex, FromHex};
use rustc_serialize::base64::{ToBase64, FromBase64, STANDARD};
use std::str;
use std::io::BufReader;
use std::io::BufWriter;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::cmp;
use std::fs::OpenOptions;


pub struct CipherText(Vec<u8>);

impl CipherText {
    pub fn new(bytes: &[u8]) -> CipherText {
        CipherText(bytes.to_vec())
    }

    pub fn from_hex(hex: &str) -> Result<CipherText, Error> {
        let cypher_bytes: Vec<u8> = try!(hex.from_hex()
            .map_err(|e| Error::HexErr(e.to_string())));
        Ok(CipherText(cypher_bytes))
    }

    pub fn to_hex(&self) -> String {
        let &CipherText(ref vec_bytes) = self;
        vec_bytes.to_hex()
    }

    pub fn from_b64(b64: &str) -> Result<CipherText, Error> {
        let cypher_bytes: Vec<u8> = try!(b64.from_base64()
            .map_err(|e| Error::Base64Err(e.to_string())));
        Ok(CipherText(cypher_bytes))
    }

    pub fn to_b64(&self) -> String {
        let &CipherText(ref vec_bytes) = self;
        vec_bytes.to_base64(STANDARD)
    }

    pub fn from_file(path: &str) -> Result<CipherText, Error> {
        let mut text = String::new();
        let mut file = try!(File::open(Path::new(path))
            .map_err(|e| Error::Base64Err(e.to_string())));
        let mut in_file = BufReader::new(file);
        for line in in_file.lines() {
            text.push_str(&line.unwrap());
        }
        CipherText::from_b64(&text)
    }

    pub fn to_file(&self, path: &str) -> Result<(), Error> {

        let text_b64 = self.to_b64();
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path)
            .unwrap();

        let mut i = 0;
        let mut j = cmp::min(text_b64.len(), 80);
        while i != j {
            file.write_all(&text_b64[i..j].as_bytes());
            file.write_all("\n".as_bytes());
            i = j;
            j = cmp::min(text_b64.len(), j + 80);
        }
        Ok(())
    }
}

pub struct PlainText(Vec<u8>);

impl PlainText {
    pub fn from_string(string: &str) -> PlainText {
        PlainText(string.as_bytes().to_vec())
    }

    pub fn from_bytes(bytes: &[u8]) -> PlainText {
        PlainText(bytes.to_vec())
    }

    pub fn from_file(path: &str) -> Result<PlainText, Error> {
        let mut file = try!(File::open(Path::new(path))
            .map_err(|e| Error::Base64Err(e.to_string())));
        let mut in_file = BufReader::new(file);
        let mut text = String::new();
        try!(in_file.read_to_string(&mut text)
            .map_err(|e| Error::Base64Err(e.to_string())));
        Ok(PlainText::from_string(&text))

    }

    pub fn to_file(&self, path: &str) -> Result<(), Error> {
        let &PlainText(ref vec_bytes) = self;
        let mut out_file = BufWriter::new(File::create(Path::new(path)).unwrap());
        out_file.write_all(&vec_bytes);
        Ok(())
    }

    pub fn to_utf8(&self) -> Result<String, Error> {
        let &PlainText(ref vec_bytes) = self;
        let plain = try!(str::from_utf8(&vec_bytes).map_err(|e| Error::UTF8Err(e.to_string())));
        Ok(plain.to_string())
    }
}

pub fn encrypt(text: &PlainText, key: &str) -> CipherText {
    let &PlainText(ref bytes) = text;
    let xored: Vec<u8> = repeating_xor(bytes, key.as_bytes());
    CipherText::new(&xored)
}

pub fn decrypt(cypher_text: &CipherText, key: &str) -> Result<PlainText, Error> {
    let &CipherText(ref bytes) = cypher_text;
    let xored: Vec<u8> = repeating_xor(&bytes, key.as_bytes());
    Ok(PlainText::from_bytes(&xored))
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &str) {
    let cipher = CipherText::from_file(input_path).unwrap();
    let plain = decrypt(&cipher, key).unwrap();
    plain.to_file(output_path);
}

pub fn encrypt_file(input_path: &str, output_path: &str, key: &str) {
    let plain = PlainText::from_file(input_path).unwrap();
    let cipher = encrypt(&plain, key);
    cipher.to_file(output_path);
}

#[derive(Debug)]
pub enum Error {
    HexErr(String),
    Base64Err(String),
    UTF8Err(String),
}

fn repeating_xor(input: &[u8], key: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(input.len());
    unsafe {
        out.set_len(input.len());
    }
    for (i, in_val) in input.iter().enumerate() {
        out[i] = in_val ^ key[i % key.len()];
    }
    return out;
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_and_decrypt() {
        let plain_text = PlainText::from_string("This is the plain text");
        let key: &str = "toy";
        assert_eq!(plain_text.to_utf8().unwrap(),
                   decrypt(&encrypt(&plain_text, key), key).unwrap().to_utf8().unwrap());
    }

}