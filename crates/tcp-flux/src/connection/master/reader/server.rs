use crate::connection::traits::RawRead;

pub struct MasterServerReader<'a, R> {
    pub(crate) reader: &'a mut R,
}

impl<'a, R: RawRead> MasterServerReader<'a, R> {}
