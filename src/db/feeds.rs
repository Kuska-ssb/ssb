use super::error::{Error, Result};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::PathBuf;

pub struct FeedsStorage {
    path: PathBuf,
}

impl FeedsStorage {
    fn filename(&self, user_id: &str) -> PathBuf {
        let name = user_id
            .chars()
            .map(|ch| match ch {
                '+' => '-',
                '/' => '_',
                _ => ch,
            })
            .collect::<String>();
        let mut path = PathBuf::new();
        path.push(&self.path);
        path.push(name);

        path
    }

    pub fn new(path: PathBuf) -> Self {
        FeedsStorage { path }
    }
    pub fn user(&self, user_id: String) -> FeedStorage {
        FeedStorage {
            path: self.filename(&user_id),
        }
    }
}

pub struct FeedStorage {
    path: PathBuf,
}

impl FeedStorage {
    /*
    raw feed storage structure:
       - last sequence in feed - 32 bits be
       - * | message-len 32 bits be
           | message
           | message-len 32 bits be
    */

    pub fn append(&self, seq_no: u32, feed: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&self.path)?;

        // check and update feed sequence number
        let created = file.seek(SeekFrom::End(0))? == 0;
        if created {
            if seq_no != 1 {
                return Err(Error::InvalidSequenceNo);
            }
        } else {
            let file_seq_no = self.get_last_seq(&mut file)?;
            if file_seq_no + 1 != seq_no {
                return Err(Error::InvalidSequenceNo);
            }
        }
        self.set_last_seq(&mut file, seq_no)?;

        file.seek(SeekFrom::End(0))?;

        // write feed size dummy
        file.write_all(&(0 as u32).to_be_bytes()[..])?;

        // write compressed feed
        let offset = file.seek(SeekFrom::Current(0))?;
        let mut wtr = snap::Writer::new(file);
        io::copy(&mut feed.as_bytes(), &mut wtr)?;

        let mut file = wtr
            .into_inner()
            .map_err(|err| Error::CompressionError(format!("{:?}", err)))?;
        let len = file.seek(SeekFrom::Current(0))? - offset;

        // write feed size
        file.write_all(&(len as u32).to_be_bytes()[..])?;

        file.seek(SeekFrom::End(-((8 + len) as i64)))?;
        file.write_all(&(len as u32).to_be_bytes()[..])?;

        Ok(())
    }

    pub fn last_seq(&self) -> Result<u32> {
        if !self.path.exists() {
            return Ok(0);
        }

        let mut file = OpenOptions::new().read(true).open(&self.path)?;
        self.get_last_seq(&mut file)
    }

    fn get_last_seq(&self, file: &mut File) -> Result<u32> {
        let mut file_seq_no = [0u8; 4];
        file.seek(SeekFrom::Start(0))?;
        file.read_exact(&mut file_seq_no[..])?;
        Ok(u32::from_be_bytes(file_seq_no))
    }

    fn set_last_seq(&self, file: &mut File, seq_no: u32) -> Result<()> {
        file.seek(SeekFrom::Start(0))?;
        file.write_all(&seq_no.to_be_bytes()[..])?;
        Ok(())
    }

    pub fn iter(&self) -> Result<FeedStorageIterator> {
        let mut file = OpenOptions::new().read(true).open(&self.path)?;

        let last_seq_no = self.get_last_seq(&mut file)?;

        Ok(FeedStorageIterator {
            file,
            current_seq_no: 0,
            last_seq_no,
        })
    }

    pub fn rev_iter(&self) -> Result<FeedStorageReverseIterator> {
        let mut file = OpenOptions::new().read(true).open(&self.path)?;

        let last_seq_no = self.get_last_seq(&mut file)?;
        file.seek(SeekFrom::End(-4))?;

        Ok(FeedStorageReverseIterator {
            file,
            current_seq_no: last_seq_no,
        })
    }
}

#[derive(PartialEq, Debug)]
pub struct Feed {
    pub seq_no: u32,
    pub value: String,
}

pub struct FeedStorageIterator {
    file: File,
    current_seq_no: u32,
    last_seq_no: u32,
}

impl Iterator for FeedStorageIterator {
    type Item = Result<Feed>;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_seq_no >= self.last_seq_no {
            return None;
        }

        // read compressed size
        let mut size_buf = [0u8; 4];
        if let Err(err) = self.file.read_exact(&mut size_buf[..]) {
            return Some(Err(Error::Io(err)));
        }
        let size = u32::from_be_bytes(size_buf);

        // read compressed data
        let mut compressed = vec![0; size as usize];
        if let Err(err) = self.file.read_exact(&mut compressed[..]) {
            return Some(Err(Error::Io(err)));
        }

        let mut rdr = snap::Reader::new(&compressed[..]);
        let mut plaintext: Vec<u8> = Vec::new();

        if let Err(err) = io::copy(&mut rdr, &mut plaintext) {
            return Some(Err(Error::Io(err)));
        }

        // read compresed size again
        if let Err(err) = self.file.read_exact(&mut size_buf[..]) {
            return Some(Err(Error::Io(err)));
        }
        if u32::from_be_bytes(size_buf) != size {
            return Some(Err(Error::MismatchReadingSecondSize));
        }

        self.current_seq_no += 1;
        let ret = match String::from_utf8(plaintext) {
            Err(err) => Err(Error::Utf8(err)),
            Ok(value) => Ok(Feed {
                seq_no: self.current_seq_no,
                value,
            }),
        };
        Some(ret)
    }
}

pub struct FeedStorageReverseIterator {
    file: File,
    current_seq_no: u32,
}

impl Iterator for FeedStorageReverseIterator {
    type Item = Result<Feed>;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_seq_no == 0 {
            return None;
        }

        // read compressed size
        let mut size_buf = [0u8; 4];
        if let Err(err) = self.file.read_exact(&mut size_buf[..]) {
            return Some(Err(Error::Io(err)));
        }
        let size = u32::from_be_bytes(size_buf);
        if let Err(err) = self.file.seek(SeekFrom::Current(-((size + 8) as i64))) {
            return Some(Err(Error::Io(err)));
        }

        // read compresed size again
        if let Err(err) = self.file.read_exact(&mut size_buf[..]) {
            return Some(Err(Error::Io(err)));
        }
        if u32::from_be_bytes(size_buf) != size {
            return Some(Err(Error::MismatchReadingSecondSize));
        }

        // read compressed data
        let mut compressed = vec![0u8; size as usize];
        if let Err(err) = self.file.read_exact(&mut compressed[..]) {
            return Some(Err(Error::Io(err)));
        }

        let mut rdr = snap::Reader::new(&compressed[..]);
        let mut plaintext: Vec<u8> = Vec::new();

        if let Err(err) = io::copy(&mut rdr, &mut plaintext) {
            return Some(Err(Error::Io(err)));
        }
        // prepare offset for the next read
        if let Err(err) = self.file.seek(SeekFrom::Current(-((size + 8) as i64))) {
            return Some(Err(Error::Io(err)));
        }

        let ret = match String::from_utf8(plaintext) {
            Err(err) => Err(Error::Utf8(err)),
            Ok(value) => Ok(Feed {
                seq_no: self.current_seq_no,
                value,
            }),
        };

        self.current_seq_no -= 1;

        Some(ret)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::iter;

    fn rand_folder() -> Result<PathBuf> {
        let mut rng = thread_rng();
        let name: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(12)
            .collect();

        let mut tmp_folder = std::env::temp_dir();
        tmp_folder.push(name);

        std::fs::create_dir(&tmp_folder)?;

        Ok(tmp_folder)
    }

    #[test]
    fn test_db_feeds() -> Result<()> {
        let user_id = "@ZFWw+UclcUgYi081/C8lhgH+KQ9s7YJRoOYGnzxW/JQ=.ed25519";
        let feeds = FeedsStorage::new(rand_folder()?);
        let feed = feeds.user(user_id.to_owned());

        let f1 = Feed {
            seq_no: 1,
            value: "123".to_string(),
        };
        let f2 = Feed {
            seq_no: 2,
            value: "8181".to_string(),
        };
        let f3 = Feed {
            seq_no: 3,
            value: "182881".to_string(),
        };

        assert_eq!(0, feed.last_seq()?);
        feed.append(f1.seq_no, &f1.value)?;
        assert_eq!(1, feed.last_seq()?);
        feed.append(f2.seq_no, &f2.value)?;
        assert_eq!(2, feed.last_seq()?);
        feed.append(f3.seq_no, &f3.value)?;
        assert_eq!(3, feed.last_seq()?);

        let mut it = feed.iter()?;
        assert_eq!(it.next().unwrap()?, f1);
        assert_eq!(it.next().unwrap()?, f2);
        assert_eq!(it.next().unwrap()?, f3);
        assert_eq!(it.next().is_none(), true);

        let mut rev_it = feed.rev_iter()?;
        assert_eq!(rev_it.next().unwrap()?, f3);
        assert_eq!(rev_it.next().unwrap()?, f2);
        assert_eq!(rev_it.next().unwrap()?, f1);
        assert_eq!(rev_it.next().is_none(), true);

        Ok(())
    }
}
