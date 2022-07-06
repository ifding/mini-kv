use serde::{Serialize, Deserialize};
use serde_repr::*;
use super::error::{KvError, Result};

const STORAGE_FILE_PREFIX: &str = "miniKV"; 
const COMPACTION_THRESHOLD: u64 = 1 << 16;
const USIZE_LEN : usize = std::mem::size_of::<usize>();
const ENTRY_HEAD_LEN : usize = USIZE_LEN * 2 + 1;

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq)]
#[repr(u8)]
pub enum CmdType {
    PUT = 1,
    DEL = 2,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    key_len: usize,
    value_len: usize,
    key: String,
    value: String,
    cmd_type: CmdType,
}

impl Entry {
    pub fn new(key: String, value: String, cmd_type: CmdType) -> Entry {
        Entry {
            key_len: key.len(),
            value_len: value.len(),
            key: key,
            value: value,
            cmd_type: cmd_type,
        }
    }

    pub fn size(&self) -> usize {
        ENTRY_HEAD_LEN + self.key_len + self.value_len
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = vec![0; self.size()];
        // encode key_len
        buf[0..USIZE_LEN]
            .copy_from_slice(&self.key_len.to_be_bytes());

        // encode value_len
        buf[USIZE_LEN..USIZE_LEN * 2]
            .copy_from_slice(&self.value_len.to_be_bytes());

        // encode cmd_type
        buf[USIZE_LEN * 2..ENTRY_HEAD_LEN]
            .copy_from_slice(bincode::serialize(&self.cmd_type).unwrap().as_slice());

        // encode key
        buf[ENTRY_HEAD_LEN..ENTRY_HEAD_LEN + self.key_len]
            .copy_from_slice(self.key.as_bytes());

        // encode value
        buf[ENTRY_HEAD_LEN + self.key_len..]
            .copy_from_slice(self.value.as_bytes());

        buf
    }

    pub fn decode(buf: &[u8; ENTRY_HEAD_LEN]) -> Result<Entry> {
        let key_len = usize::from_be_bytes(buf[0..USIZE_LEN].try_into()?);
        let value_len = usize::from_be_bytes(buf[USIZE_LEN..USIZE_LEN * 2].try_into()?);
        let cmd_type = bincode::deserialize(&buf[USIZE_LEN * 2..ENTRY_HEAD_LEN])?;
        Ok(Entry {
            key_len: key_len,
            value_len: value_len,
            key: String::new(),
            value: String::new(),
            cmd_type: cmd_type,
        })
    }
}

pub trait Storage {
    fn get(&mut self, key: String) -> Result<Option<String>>;
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn remove(&mut self, key: String) -> Result<()>;
}

use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::io::{Read, Write, Seek, SeekFrom, BufReader, BufWriter};
use std::collections::HashMap;
pub struct Bitcask {
    path_buf: PathBuf,
    reader: BufReaderPos<File>,
    writer: BufWriterPos<File>,
    index: HashMap<String, u64>,
    compaction: u64,
}

impl Storage for Bitcask {
    fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.read(&key) {
            Ok(entry) => Ok(Some(entry.value)),
            Err(KvError::KeyNotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn set(&mut self, key: String, value: String) -> Result<()> {
        let entry = Entry::new(key, value, CmdType::PUT);
        self.write(entry)?;
        if self.compaction >= COMPACTION_THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }

    fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let entry = Entry::new(key.clone(), String::new(), CmdType::DEL);
            self.write(entry)?;
            self.index.remove(&key);
            return Ok(());
        }
        Err(KvError::KeyNotFound)
    }
}

impl Bitcask {
    pub fn open(path_buf: PathBuf) -> Result<Bitcask> {
        let data_path = path_buf.join(STORAGE_FILE_PREFIX.to_string() + ".data");
        let writer = BufWriterPos::new( 
            OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(data_path.as_path())?
        )?;
        let reader = BufReaderPos::new(File::open(data_path.as_path())?)?;

        let mut instance = Bitcask {
            path_buf: path_buf,
            reader: reader,
            writer: writer,
            index: HashMap::new(),
            compaction: 0,
        };
        instance.load_index()?;
        Ok(instance)
    }

    fn write(&mut self, entry: Entry) -> Result<()> {
        let key = entry.key.clone();
        if let Some(pos) = self.index.insert(key, self.writer.pos) {
            self.compaction += self.read_at(pos).unwrap().size() as u64;
        } 
        let buf = entry.encode();
        self.writer.write(&buf)?;
        self.writer.flush()?;
        Ok(())
    }

    fn read(&mut self, key:&str) -> Result<Entry> {
        if let Some(offset) = self.index.get(key) {
            return self.read_at(*offset);
        }
        Err(KvError::KeyNotFound)
    }

    fn read_at(&mut self, offset: u64) -> Result<Entry> {
        self.reader.seek(SeekFrom::Start(offset))?;
        let mut buf: [u8; ENTRY_HEAD_LEN] = [0; ENTRY_HEAD_LEN];
        let len = self.reader.read(&mut buf)?;
        if len == 0 {
            return Err(KvError::EOF);
        }
        let mut entry = Entry::decode(&buf)?;
        let mut key_buf = vec![0; entry.key_len];
        self.reader.read_exact(key_buf.as_mut_slice())?;
        entry.key = String::from_utf8(key_buf)?;

        let mut value_buf = vec![0; entry.value_len];
        self.reader.read_exact(value_buf.as_mut_slice())?;
        entry.value = String::from_utf8(value_buf)?;

        Ok(entry)
    }

    fn load_index(&mut self) -> Result<()> {
        let mut offset = 0;
        loop {
            match self.read_at(offset) {
                Ok(entry) => {
                    if entry.cmd_type == CmdType::DEL {
                        self.index.remove(&entry.key);
                        continue;
                    }
                    let size = entry.size() as u64;
                    self.index.insert(entry.key, offset);
                    offset += size;
                },
                Err(KvError::EOF) => {
                    self.writer.pos = offset;
                    break;
                },
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn compact(&mut self) -> Result<()> {
        let mut new_entry = Vec::new();
        let mut offset = 0;
        loop {
            match self.read_at(offset) {
                Ok(entry) => {
                    let size = entry.size() as u64;
                    if let Some(pos) = self.index.get(&entry.key) {
                        if entry.cmd_type == CmdType::DEL && *pos == offset {
                            new_entry.push(entry);
                        }
                    }
                    offset += size;
                },
                Err(KvError::EOF) => break,
                Err(e) => return Err(e),
            }
        }
        if !new_entry.is_empty() {
            let mut data_buf_ancestors = self.path_buf.ancestors();
            data_buf_ancestors.next();
            let new_path_buf = data_buf_ancestors
                .next()
                .ok_or(KvError::InvalidDataPath)?
                .join(STORAGE_FILE_PREFIX.to_string() + ".compact");
            let mut write_buf = BufWriterPos::new(File::create(new_path_buf.as_path())?)?;

            for entry in &new_entry {
                let key = entry.key.clone();
                self.index.insert(key, write_buf.pos);
                write_buf.write(&entry.encode())?;
            }

            self.writer = write_buf;
            self.reader = BufReaderPos::new(File::open(new_path_buf.as_path())?)?;
            std::fs::remove_file(self.path_buf.as_path())?;
            std::fs::rename(new_path_buf, self.path_buf.as_path())?;
        }

        self.compaction = 0;
        Ok(())
    }
}

use std::io;

struct BufReaderPos<R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Read + Seek> BufReaderPos<R> {
    fn new(mut reader: R) -> Result<Self> {
        let pos = reader.seek(SeekFrom::Current(0))?;
        Ok(BufReaderPos {
            reader: BufReader::new(reader),
            pos: pos,
        })
    }
}

impl<R: Read + Seek> Read for BufReaderPos<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

impl<R: Read + Seek> Seek for BufReaderPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

struct BufWriterPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64,
}

impl<W: Write + Seek> BufWriterPos<W> {
    fn new(mut writer: W) -> Result<Self> {
        let pos = writer.seek(SeekFrom::Current(0))?;
        Ok(BufWriterPos {
            writer: BufWriter::new(writer),
            pos: pos,
        })
    }
}

impl<W: Write + Seek> Write for BufWriterPos<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for BufWriterPos<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}