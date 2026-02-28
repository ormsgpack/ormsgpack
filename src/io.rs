// SPDX-License-Identifier: (Apache-2.0 OR MIT)

pub trait Read {
    fn read_array<const N: usize>(&mut self) -> Result<&[u8; N], std::io::Error>;
    fn read_slice(&mut self, len: usize) -> Result<&[u8], std::io::Error>;

    #[inline(always)]
    fn read_f32(&mut self) -> Result<f32, std::io::Error> {
        let bytes = self.read_array()?;
        Ok(f32::from_be_bytes(*bytes))
    }

    #[inline(always)]
    fn read_f64(&mut self) -> Result<f64, std::io::Error> {
        let bytes = self.read_array()?;
        Ok(f64::from_be_bytes(*bytes))
    }

    #[inline(always)]
    fn read_i8(&mut self) -> Result<i8, std::io::Error> {
        let bytes: &[u8; 1] = self.read_array()?;
        Ok(bytes[0] as i8)
    }

    #[inline(always)]
    fn read_i16(&mut self) -> Result<i16, std::io::Error> {
        let bytes = self.read_array()?;
        Ok(i16::from_be_bytes(*bytes))
    }

    #[inline(always)]
    fn read_i32(&mut self) -> Result<i32, std::io::Error> {
        let bytes = self.read_array()?;
        Ok(i32::from_be_bytes(*bytes))
    }

    #[inline(always)]
    fn read_i64(&mut self) -> Result<i64, std::io::Error> {
        let bytes = self.read_array()?;
        Ok(i64::from_be_bytes(*bytes))
    }

    #[inline(always)]
    fn read_u8(&mut self) -> Result<u8, std::io::Error> {
        let bytes: &[u8; 1] = self.read_array()?;
        Ok(bytes[0])
    }

    #[inline(always)]
    fn read_u16(&mut self) -> Result<u16, std::io::Error> {
        let bytes = self.read_array()?;
        Ok(u16::from_be_bytes(*bytes))
    }

    #[inline(always)]
    fn read_u32(&mut self) -> Result<u32, std::io::Error> {
        let bytes = self.read_array()?;
        Ok(u32::from_be_bytes(*bytes))
    }

    #[inline(always)]
    fn read_u64(&mut self) -> Result<u64, std::io::Error> {
        let bytes = self.read_array()?;
        Ok(u64::from_be_bytes(*bytes))
    }
}

impl Read for &[u8] {
    fn read_array<const N: usize>(&mut self) -> Result<&[u8; N], std::io::Error> {
        let (a, b) = match self.split_first_chunk() {
            Some(value) => value,
            None => return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof)),
        };
        *self = b;
        Ok(a)
    }

    fn read_slice(&mut self, len: usize) -> Result<&[u8], std::io::Error> {
        let (a, b) = match self.split_at_checked(len) {
            Some(value) => value,
            None => return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof)),
        };
        *self = b;
        Ok(a)
    }
}

pub trait WriteSlices: std::io::Write {
    fn write_slices<const N: usize>(&mut self, bufs: [&[u8]; N]) -> Result<(), std::io::Error>;
}

impl<T> WriteSlices for &mut T
where
    T: WriteSlices,
{
    fn write_slices<const N: usize>(&mut self, bufs: [&[u8]; N]) -> Result<(), std::io::Error> {
        (**self).write_slices(bufs)
    }
}
