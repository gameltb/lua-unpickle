use serde_json::{Map, Number, Value};
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, BufReader, SeekFrom};

#[derive(Debug)]
struct LuaPickle {
    file: BufReader<File>,
}

enum Strsize {
    Byte,
    Halfword,
}

impl LuaPickle {
    fn read(&mut self) -> Result<(u8, Value), Box<dyn std::error::Error>> {
        let mut type_char = [0u8; 1];
        self.file.read_exact(&mut type_char).unwrap();

        let mut type_char = type_char[0];
        let content = match type_char {
            0x00 => Value::default(),
            0x01 => Value::Bool(true),
            0x02 => Value::Bool(false),
            0x03 => {
                let mut buff = [0u8; 1];
                self.file.read_exact(&mut buff).unwrap();
                Value::Number(Number::from(buff[0]))
            }
            0x04 => {
                let mut buff = [0u8; 2];
                self.file.read_exact(&mut buff).unwrap();
                let v = u16::from_le_bytes(buff);
                Value::Number(Number::from(v))
            }
            0x05 => {
                let mut buff = [0u8; 4];
                self.file.read_exact(&mut buff).unwrap();
                let v = u32::from_le_bytes(buff);
                Value::Number(Number::from(v))
            }
            0x06 => {
                let mut buff = [0u8; 8];
                self.file.read_exact(&mut buff).unwrap();
                let v = u64::from_le_bytes(buff);
                Value::Number(Number::from(v))
            }
            0x07 => {
                let mut buff = [0u8; 8];
                self.file.read_exact(&mut buff).unwrap();
                let v = f64::from_le_bytes(buff);
                Value::Number(Number::from_f64(v).ok_or("not valid 64bit float").unwrap())
            }
            0x08 => self.readstr(Strsize::Byte).unwrap(),
            0x09 => self.readstr(Strsize::Halfword).unwrap(),
            0x0b => {
                let mut buff = [0u8; 4];
                self.file.read_exact(&mut buff).unwrap();
                let count = u32::from_le_bytes(buff);
                self.readtab(count).unwrap()
            }
            0x0d => {
                let (_, key) = self.read().unwrap();
                let (_, content) = self.read().unwrap();
                let mut dict: Map<String, Value> = Map::new();
                dict.insert(key.to_string(), content);
                Value::Object(dict)
            }
            0x0e => {
                let mut buff = [0u8; 1];
                self.file.read_exact(&mut buff).unwrap();
                Value::Number(Number::from(buff[0]))
            }
            0x0f => {
                let mut buff = [0u8; 2];
                self.file.read_exact(&mut buff).unwrap();
                let v = u16::from_le_bytes(buff);
                Value::Number(Number::from(v))
            }
            0x10 => {
                let mut buff = [0u8; 4];
                self.file.read_exact(&mut buff).unwrap();
                let v = u32::from_le_bytes(buff);
                Value::Number(Number::from(v))
            }
            _ => {
                let pos = self.file.seek(SeekFrom::Current(0)).unwrap();
                panic!("unhandle type:{} pos:{}", type_char, pos);
                type_char = 0;
                Value::default()
            }
        };
        //println!("{} => {:?}",type_char,content);
        Ok((type_char, content))
    }
    fn readtab(&mut self, count: u32) -> Result<Value, Box<dyn std::error::Error>> {
        let mut dict: Map<String, Value> = Map::new();
        if count > 0 {
            for index in 1..count + 1 {
                let (_, coutent) = self.read().unwrap();
                dict.insert(index.to_string(), coutent);
            }
        }
        loop {
            let (type_char, key) = self.read().unwrap();
            if type_char == 0 {
                break;
            }
            let (_, coutent) = self.read().unwrap();
            let k = match key {
                Value::Number(_) => key.as_u64().ok_or("cast fail").unwrap().to_string(),
                Value::String(_) => key.as_str().ok_or("cast fail").unwrap().to_string(),
                _ => key.to_string(),
            };
            dict.insert(k, coutent);
        }
        Ok(Value::Object(dict))
    }
    fn readstr(&mut self, fmt: Strsize) -> std::io::Result<Value> {
        let strlen: u32 = match fmt {
            Strsize::Byte => {
                let mut buff = [0u8; 1];
                self.file.read_exact(&mut buff).unwrap();
                buff[0] as u32
            }
            Strsize::Halfword => {
                let mut buff = [0u8; 2];
                self.file.read_exact(&mut buff).unwrap();
                u16::from_le_bytes(buff) as u32
            }
        };

        let mut string = String::new();
        self.file
            .by_ref()
            .take(strlen as u64)
            .read_to_string(&mut string).unwrap();
        Ok(Value::String(string))
    }
}

pub fn unpickle(filepath: &str, skipbyte: u64) -> Result<Value, Box<dyn std::error::Error>> {
    let file = OpenOptions::new().read(true).open(filepath).unwrap();

    let mut file = BufReader::new(file);

    file.seek(SeekFrom::Start(skipbyte)).unwrap();

    let mut an_pickle = LuaPickle { file };
    let (_, dict) = an_pickle.read().unwrap();

    Ok(dict)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_read() {
        let filepath = "cache";
        let tabs = super::unpickle(filepath, 4).unwrap();
        let filepath = std::path::PathBuf::from(filepath);
        for (name, tab) in tabs.as_object().unwrap() {
            let mut file = std::fs::File::create(
                filepath
                    .parent()
                    .or(Some(std::env::current_dir().unwrap().as_path()))
                    .unwrap()
                    .clone()
                    .join(name.to_owned() + ".json")
            )
            .unwrap();
            use std::io::Write;
            file.write(tab.to_string().as_bytes()).unwrap();
        }
    }
}
