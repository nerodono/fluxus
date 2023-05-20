use std::io::{
    self,
    IoSlice,
};

use crate::traits::ComposeWrite;

pub async fn write_two_bufs<W: ComposeWrite>(
    raw: &mut W,
    mut prepend: &[u8],
    append: &[u8],
) -> io::Result<()> {
    let prepend_len = prepend.len();
    let append_len = append.len();
    let total = append_len + prepend_len;

    if !raw.is_write_vectored() {
        // Since write is not vectored we'll just create
        // intermediate buffer
        let mut vec = Vec::with_capacity(total);
        vec.extend(prepend.iter().copied());
        vec.extend(append.iter().copied());

        return raw.write_all(&vec).await;
    }

    let mut ios = [IoSlice::new(prepend), IoSlice::new(append)];
    let mut sent = 0_usize;
    loop {
        let cur_wrote @ 1.. = raw.write_vectored(&ios).await? else {
            break Err(io::Error::last_os_error());
        };
        sent += cur_wrote;

        if sent >= total {
            break Ok(());
        }
        // sent < total
        if sent >= prepend_len {
            // then prepend buffer was fully wrote
            // we write the remaining into the single system call
            // (ideally)
            return raw
                .write_all(&append[(sent - prepend_len)..])
                .await;
        }

        // sent < prepend_len
        // then we should remove written to stream bytes
        prepend = &prepend[cur_wrote..];
        ios[0] = IoSlice::new(prepend);
    }
}
