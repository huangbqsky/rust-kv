use crate::Command;
use crate::KVStoreError::KeyNotFound;
use crate::{KVStoreError, Result};
use serde_json::Deserializer;
use std::collections::HashMap;
use std::fs::{create_dir_all, read_dir, remove_file, File, OpenOptions};
use std::io;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

const MAX_USELESS_SIZE: u64 = 1024;

/** A KvStore stores key/value pairs in HashMap.
# Example
```
use std::env;
use kvs::{KvStore, Result};
# fn try_main() -> Result<()> {
    let mut store = KvStore::open(env::current_dir()?)?;
    store.set("1".to_owned(),"1".to_owned())?;
    assert_eq!(store.get("1".to_owned())?, Some("1".to_owned()));
    store.remove("1".to_owned())?;
    assert_eq!(store.get("1".to_owned())?, None);
    Ok(())
# }
```
 */

/**
KvStore 结构体中各个变量含义如下：
Index ：参照 bitcask 的模型，key 为 kv pair 的 key，value 并不存储对应的 value，而是存储该 value 在第 file_number 个文件的 offset 处，长度为 length。
current_readers：对于所有已经存在的文件，KvStore 都缓存了一个 BufReader 来便于 seek 到对应的 offset 去 read。实际上也可以没有该结构体每次需要 reader 时新建即可，但复用 reader 可以一定程度上减少资源的损耗。
current_writer：当前正在写入的 file，其每次写入只需要 append 即可，不需要 seek。新建一个 BufWriterWithPosition 结构体的原因是能够快速的获取当前写入的 offset，而不需要在通过 seek(SeekFrom::Current(0))（可能是系统调用） 的方式去获取。
current_file_number：当前最大的 file_number，每次 compaction 之后会新增 1。每个数据文件都会附带一个 file_number，file_number 越大的文件越新，该 version 能够保证恢复时的正确性。
dir_path：当前文件目录路径。
useless_size：当前无用的数据总和。当改值大于某一个阈值时，会触发一次 compaction。
 */
pub struct KvStore {
    index: HashMap<String, CommandPosition>,
    current_readers: HashMap<u64, BufReader<File>>,
    current_writer: BufWriterWithPosition<File>,
    current_file_number: u64,
    dir_path: PathBuf,
    useless_size: u64,
}

impl KvStore {
    /// 打开 Open the KvStore at a given path. Return the KvStore.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let dir_path = path.into();
        create_dir_all(&dir_path)?;

        let mut index = HashMap::new();

        let mut current_readers = HashMap::new();

        let (current_file_number, useless_size) =
            Self::recover(&dir_path, &mut current_readers, &mut index)?;

        let current_file_path = dir_path.join(format!("data_{}.txt", current_file_number));

        let current_writer = BufWriterWithPosition::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&current_file_path)?,
        )?;

        if current_file_number == 0 {
            current_readers.insert(
                current_file_number,
                BufReader::new(File::open(&current_file_path)?),
            );
        }

        let mut store = KvStore {
            index,
            current_readers,
            current_writer,
            current_file_number,
            dir_path,
            useless_size,
        };

        if store.useless_size > MAX_USELESS_SIZE {
            store.compact()?;
        }

        Ok(store)
    }

    /// 设置 Set the value of a string key to a string. Return an error if the value is not written successfully.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let command = Command::SET(key, value);
        let data = serde_json::to_vec(&command)?;

        let offset = self.current_writer.get_position();
        self.current_writer.write_all(&data)?;
        self.current_writer.flush()?;
        let length = self.current_writer.get_position() - offset;
        let file_number = self.current_file_number;

        if let Command::SET(key, _) = command {
            self.useless_size += self
                .index
                .insert(
                    key,
                    CommandPosition {
                        offset,
                        length,
                        file_number,
                    },
                )
                .map(|cp| cp.length)
                .unwrap_or(0);
        }

        if self.useless_size > MAX_USELESS_SIZE {
            self.compact()?;
        }

        Ok(())
    }

    /// 获取 Get the string value of a string key. If the key does not exist, return None. Return an error if the value is not read successfully.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(position) = self.index.get(&key) {
            let source_reader = self
                .current_readers
                .get_mut(&position.file_number)
                .expect("Can not find key in files but it is in memory");
            source_reader.seek(SeekFrom::Start(position.offset))?;
            let data_reader = source_reader.take(position.length as u64);

            if let Command::SET(_, value) = serde_json::from_reader(data_reader)? {
                Ok(Some(value))
            } else {
                Err(KVStoreError::UnknownCommandType)
            }
        } else {
            Ok(None)
        }
    }

    /// 删除 Remove a given key. Return an error if the key does not exist or is not removed successfully.
    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.index.get(&key).is_some() {
            self.useless_size += self.index.remove(&key).map(|cp| cp.length).unwrap_or(0);

            let command = serde_json::to_vec(&Command::RM(key))?;
            let offset = self.current_writer.get_position();
            self.current_writer.write_all(&command)?;
            self.current_writer.flush()?;

            self.useless_size += self.current_writer.get_position() - offset;

            if self.useless_size > MAX_USELESS_SIZE {
                self.compact()?;
            }

            Ok(())
        } else {
            Err(KeyNotFound)
        }
    }

    fn create_new_file(&mut self) -> Result<()> {
        self.current_file_number += 1;
        let new_file_path = self
            .dir_path
            .join(format!("data_{}.txt", self.current_file_number));
        self.current_writer = BufWriterWithPosition::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&new_file_path)?,
        )?;
        self.current_readers.insert(
            self.current_file_number,
            BufReader::new(File::open(new_file_path)?),
        );

        Ok(())
    }

    // 重启
    fn recover(
        dir_path: &PathBuf,
        current_readers: &mut HashMap<u64, BufReader<File>>,
        index: &mut HashMap<String, CommandPosition>,
    ) -> Result<(u64, u64)> {
        let mut versions: Vec<u64> = read_dir(dir_path)?
            .flat_map(|res| res.map(|e| e.path()))
            .filter(|path| path.is_file() && path.extension() == Some("txt".as_ref()))
            .flat_map(|path| {
                path.file_name()
                    .and_then(|filename| filename.to_str())
                    .map(|filename| {
                        filename
                            .trim_start_matches("data_")
                            .trim_end_matches(".txt")
                    })
                    .map(str::parse::<u64>)
            })
            .flatten()
            .collect();
        versions.sort();

        let mut useless_size = 0;
        for version in &versions {
            let file_path = dir_path.join(format!("data_{}.txt", version));
            let reader = BufReader::new(File::open(&file_path)?);
            let mut iter = Deserializer::from_reader(reader).into_iter::<Command>();
            let mut before_offset = iter.byte_offset() as u64;
            while let Some(command) = iter.next() {
                let after_offset = iter.byte_offset() as u64;
                match command? {
                    Command::SET(key, _) => {
                        useless_size += index
                            .insert(
                                key,
                                CommandPosition {
                                    offset: before_offset,
                                    length: after_offset - before_offset,
                                    file_number: *version,
                                },
                            )
                            .map(|cp| cp.length)
                            .unwrap_or(0);
                    }
                    Command::RM(key) => {
                        useless_size += index.remove(&key).map(|cp| cp.length).unwrap_or(0);
                        useless_size += after_offset - before_offset;
                    }
                };
                before_offset = after_offset;
            }
            current_readers.insert(*version, BufReader::new(File::open(&file_path)?));
        }

        Ok((*versions.last().unwrap_or(&0), useless_size))
    }

    // 合并
    fn compact(&mut self) -> Result<()> {
        self.create_new_file()?;

        let mut before_offset = 0;
        for position in self.index.values_mut() {
            let source_reader = self
                .current_readers
                .get_mut(&position.file_number)
                .expect("Can not find key in files but it is in memory");
            source_reader.seek(SeekFrom::Start(position.offset))?;
            let mut data_reader = source_reader.take(position.length);
            io::copy(&mut data_reader, &mut self.current_writer)?;
            let after_offset = self.current_writer.position;
            *position = CommandPosition {
                offset: before_offset,
                length: after_offset - before_offset,
                file_number: self.current_file_number,
            };
            before_offset = after_offset;
        }
        self.current_writer.flush()?;

        let delete_file_numbers: Vec<u64> = self
            .current_readers
            .iter()
            .map(|(key, _)| *key)
            .filter(|key| *key < self.current_file_number)
            .collect();

        for number in delete_file_numbers {
            self.current_readers.remove(&number);
            remove_file(self.dir_path.join(format!("data_{}.txt", number)))?;
        }

        self.create_new_file()?;

        Ok(())
    }
}

/// 一个记录写入位置的结构体 a struct which records writer's current position
struct BufWriterWithPosition<T: Write + Seek> {
    position: u64,
    writer: BufWriter<T>,
}

impl<T: Write + Seek> BufWriterWithPosition<T> {
    fn new(mut inner: T) -> Result<Self> {
        let position = inner.seek(SeekFrom::End(0))?;
        Ok(BufWriterWithPosition {
            position,
            writer: BufWriter::new(inner),
        })
    }

    fn get_position(&self) -> u64 {
        self.position
    }
}

impl<T: Write + Seek> Write for BufWriterWithPosition<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.position += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// 一个记录命令行信息的结构体 a struct which records command's metadata
struct CommandPosition {
    offset: u64,
    length: u64,
    file_number: u64,
}