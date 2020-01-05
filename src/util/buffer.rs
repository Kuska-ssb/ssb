use std::cmp; 
use std::borrow::Cow;
use std::io::{Read,Write,self};
use async_std::{
    pin::Pin,
    task::{Context,Poll}
};
use log::debug;

#[derive(Debug)]
pub struct CircularBuffer {
    buffer: Box<[u8]>,
    start: usize, 
    end:   usize,
    len:   usize,
}

impl CircularBuffer {
    pub fn new(capacity : usize) -> CircularBuffer {
        CircularBuffer {
            buffer: vec![0; capacity].into_boxed_slice(),
            start: 0,
            end: 0,
            len: 0,
        }
    }
    pub fn to_string(&self) -> Cow<str> {
        if self.len == 0 {
            Cow::Borrowed("")
        } else {
            let mut s = String::new();
            if self.start < self.end {
                s.push_str(&hex::encode(&self.buffer[self.start..self.end]));    
            } else {
                s.push_str(&hex::encode(&self.buffer[self.start..]));    
                s.push_str(&hex::encode(&self.buffer[..self.end]));
            }
            Cow::Owned(s)
        }
    }
    
    pub fn len(&self) -> usize {
        return self.len;
    }
    pub fn cap(&self) -> usize {
        return self.buffer.len();
    }
    pub fn clear(&mut self) {
        self.start = 0;
        self.end = 0;
        self.len = 0;
    }
    pub fn write_from<R: Read>(&mut self, reader : &mut R) -> io::Result<()> {        
        let mut readed = 1;

        loop {
            if readed==0 || self.len == self.buffer.len() {
                return Ok(())
            } else if self.start <= self.end {
                // write at %end -> end_buffer 
                readed = reader.read(&mut self.buffer[self.end..])?;
                self.end = (readed + self.end) % self.buffer.len();
            } else {
                // write at %end -> %start
                readed = reader.read(&mut self.buffer[self.end..self.start])?;
                self.end += readed;
            }
            self.len += readed;
        }
    }

    pub fn defrag(&mut self) {
        if self.len > 0 {
            if self.start < self.end && self.start > 0 {
                self.buffer.copy_within(self.start..self.end, 0);
            } else if self.end <= self.start && !(self.start==self.end && self.start==0)  {
                let mut tail = Vec::with_capacity(self.end);
                tail.resize(self.end, 0);
                &tail[..].copy_from_slice(&self.buffer[..self.end]);
                self.buffer.copy_within(self.start..self.buffer.len(), 0);
                &self.buffer[self.end..self.end+tail.len()].copy_from_slice(&tail[..]);
            } 
        }
        self.start = 0;
        self.end = self.len();
    }

    pub fn contiguous_value(&mut self) -> &[u8] {
        if self.len == 0 {
            &self.buffer[0..0]
        } else if self.start < self.end {
            &self.buffer[self.start..self.end]
        } else  {
            &self.buffer[self.start..]
        }
    }

    pub fn contiguous_free(&mut self) -> &[u8] {
        if self.len == self.buffer.len() {
            &self.buffer[0..0]
        } else if self.start <= self.end {
            &self.buffer[self.end..]
        } else  {
            &self.buffer[self.end..self.start]
        }
    }

    pub fn skip(&mut self, n: usize) {
        let n = cmp::min(n,self.len());
        self.start = (self.start + n) % self.buffer.len();
        self.len -= n;
    }
}

impl Read for CircularBuffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len()==0 || self.len == 0 {
            Ok(0)
        } else {
            debug!("  cbuffer_read {}",buf.len());

            let last_len = self.len;

            // read from the %start to the %end or end of buffer
            let len_a = cmp::min(buf.len(), if self.start < self.end {
                self.end - self.start
            } else {
                self.buffer.len() - self.start
            });
            buf[..len_a].copy_from_slice(&self.buffer[self.start..self.start+len_a]);
            self.start = (self.start + len_a) % self.buffer.len();
            self.len -= len_a;

            // if there's somthing pending to read, the head is at the beginning
            if self.len > 0 && buf.len() > len_a {
                let len_b = cmp::min(buf.len()-len_a,self.end);
                buf[len_a..len_a+len_b].copy_from_slice(&self.buffer[..len_b]);
                self.start += len_b;
                self.len -= len_b;
            }  

            Ok(last_len - self.len)
        }
    }
}

impl async_std::io::Read for CircularBuffer {
    fn poll_read( self: Pin<&mut Self>, _cx: &mut Context, buf : &mut [u8] ) -> Poll<async_std::io::Result<usize>> {
        // TODO(amb) check
        debug!("  cbuffer_poll_read {}",buf.len());
        Poll::Ready(self.get_mut().read(buf))
    }
}

impl async_std::io::Write for CircularBuffer {
    fn poll_write(self: Pin<&mut Self>, _cx: &mut Context,buf: &[u8]) -> Poll<async_std::io::Result<usize>> {
        // TODO(amb) check
        Poll::Ready(self.get_mut().write(buf))
    }
    fn poll_flush(self: Pin<&mut Self>, _cx : &mut Context) -> Poll<async_std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
    fn poll_close(self: Pin<&mut Self>, _cx : &mut Context) -> Poll<async_std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

impl Write for CircularBuffer {

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {

        if buf.len()==0 || self.len == self.buffer.len() {
            Ok(0)
        } else {        
            let last_len = self.len;
            let mut offset = 0;

            if self.start <= self.end {
                // append after the %end, up to (maximum) the end of the buffer
                // if end of buffer is reached, %end points to the start of the buffer
                let len = cmp::min(self.buffer.len()-self.end,buf.len());
                self.buffer[self.end..self.end+len].copy_from_slice(&buf[..len]);
                self.end = (len + self.end) % self.buffer.len();
                self.len += len;
                offset = len;
            }
            
            if offset < buf.len() {
                // append from the %end to the %start   
                let len = cmp::min(self.start-self.end,buf.len()-offset);        
                self.buffer[self.end..self.end+len].copy_from_slice(&buf[offset..offset+len]);
                self.end += len;
                self.len += len;
            }

            debug!("    cbuffer_write requested {} writtern {}",buf.len(), self.len - last_len);
            Ok(self.len - last_len)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_readwrite() -> io::Result<()> {
        let mut b = super::CircularBuffer::new(6);
        assert_eq!("",b.to_string());

        // add 
        assert_eq!(5,b.write(&[1,2,3,4,5])?);
        assert_eq!("0102030405", b.to_string());
        assert_eq!("010203040500", hex::encode(&b.buffer));


        // add with overflow
        assert_eq!(1,b.write(&[6,7,8])?);
        assert_eq!("010203040506",b.to_string());
        assert_eq!("010203040506", hex::encode(&b.buffer));

        // get 4
        let mut buff = [0u8;4];
        assert_eq!(4,b.read(&mut buff[..])?);
        assert_eq!("0506",b.to_string());
        assert_eq!("01020304",hex::encode(&buff[..]));
        assert_eq!("010203040506", hex::encode(&b.buffer));

        // add 2
        assert_eq!(2,b.write(&[7,8])?);
        assert_eq!("05060708",b.to_string());
        assert_eq!("070803040506",hex::encode(&b.buffer[..]));
        
        // remove 3
        let mut buff = [0u8;3];
        assert_eq!(3,b.read(&mut buff[..])?);
        assert_eq!("08",b.to_string());
        assert_eq!("050607",hex::encode(&buff[..]));
        assert_eq!("070803040506",hex::encode(&b.buffer[..]));
        
        // add 5
        assert_eq!(5,b.write(&[9,10,11,12,13,14,15,16])?);
        assert_eq!("08090a0b0c0d",b.to_string());
        assert_eq!("0d08090a0b0c",hex::encode(&b.buffer[..]));

        // get 6
       let mut buff = [0u8;6];
       assert_eq!(6,b.read(&mut buff[..])?);
       assert_eq!("",b.to_string());
       assert_eq!("08090a0b0c0d",hex::encode(&buff[..]));
       assert_eq!("0d08090a0b0c",hex::encode(&b.buffer[..]));
        
       Ok(())
    }

    #[test]
    fn check_peek_skip() -> io::Result<()> {
        let mut b = super::CircularBuffer::new(6);
        assert_eq!("",b.to_string());
        
        // add 5
        b.write(&[1,2,3,4,5])?;
        assert_eq!("0102030405", hex::encode(b.contiguous_value()));

        // skip 4
        b.skip(4);
        assert_eq!("05", hex::encode(b.contiguous_value()));

        // add 2
        b.write(&[6,7,8])?;
        assert_eq!("0506", hex::encode(b.contiguous_value()));

        // skip 3
        b.skip(3);
        assert_eq!("08", hex::encode(b.contiguous_value()));
        
        Ok(())
    }

    #[test]
    fn check_defrag_empty() -> io::Result<()> {

        // try to defrag when is empty
        let mut b = super::CircularBuffer::new(6);
        assert_eq!(6, b.contiguous_free().len());
        b.write(&[1,2,3,4,5])?;
        assert_eq!(1, b.contiguous_free().len());
        b.skip(5);
        assert_eq!(1, b.contiguous_free().len());
        b.defrag();
        assert_eq!(6, b.contiguous_free().len());        
        Ok(())
    }

    #[test]
    fn check_defrag_end() -> io::Result<()> {
        // defrag with some data at the end
        // 
        let mut b = super::CircularBuffer::new(6);
        b.write(&[1,2,3,4,5])?;
        assert_eq!(1, b.contiguous_free().len());
        b.skip(4);
        assert_eq!(1, b.contiguous_free().len());
        b.defrag();
        assert_eq!(5, b.contiguous_free().len());
        assert_eq!("05", b.to_string());
        assert_eq!("05", hex::encode(b.contiguous_value()));
        Ok(())
    }

    #[test]
    fn check_defrag_begin() -> io::Result<()> {
        // try to defrag with empty data begib
        // 
        let mut b = super::CircularBuffer::new(6);
        assert_eq!(6, b.contiguous_free().len());
        b.write(&[1,2,3,4,5])?;
        b.skip(1);
        assert_eq!(1, b.contiguous_free().len());
        b.defrag();
        assert_eq!(2, b.contiguous_free().len());
        assert_eq!("02030405", b.to_string());
        assert_eq!("02030405", hex::encode(b.contiguous_value()));
        Ok(())
    }

    #[test]
    fn check_fuzzing() -> io::Result<()> {
        let mut r = [0u8;7];
        let mut b = super::CircularBuffer::new(r.len());
        for i in 0..10000 {            
            let n1 = (i) % b.cap();
            let n2 = (2*i) % b.cap();
            let n3 = (3*i) % b.cap();
            b.skip(n1);
            b.defrag();
            b.write(&r[..n2])?;
            b.read(&mut r[..n3])?;
            b.write(&r[..n2])?;
        }
        Ok(())
    }
}


