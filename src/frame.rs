//! One JSON object per line, both ways (PROTOCOL.md §2).

use serde_json::Value;
use tokio::io::{AsyncBufRead, AsyncBufReadExt};

/// The longest line either side reads. A longer one is skipped whole and said.
pub(crate) const MAX_LINE: usize = 16 << 20;

/// What the next line was.
pub(crate) enum Read {
    /// A line, now in the buffer, without its newline.
    Line,
    /// A line over [`MAX_LINE`], skipped.
    TooLong,
}

/// Read the next line into `line`. `None` once the stream has ended.
pub(crate) async fn next<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    line: &mut Vec<u8>,
) -> std::io::Result<Option<Read>> {
    line.clear();
    let mut over = false;
    let mut any = false;
    loop {
        let available = reader.fill_buf().await?;
        if available.is_empty() {
            return Ok(match (any, over) {
                (false, _) => None,
                (true, true) => Some(Read::TooLong),
                (true, false) => Some(Read::Line),
            });
        }
        any = true;
        let newline = available.iter().position(|byte| *byte == b'\n');
        let take = newline.unwrap_or(available.len());
        if !over {
            if line.len() + take > MAX_LINE {
                over = true;
                line.clear();
            } else {
                line.extend_from_slice(&available[..take]);
            }
        }
        reader.consume(newline.map_or(take, |at| at + 1));
        if newline.is_some() {
            return Ok(Some(if over { Read::TooLong } else { Read::Line }));
        }
    }
}

/// `message` as one line.
pub(crate) fn encode(message: &Value) -> Vec<u8> {
    // A `Value` always serializes: its map keys are strings.
    let mut line = serde_json::to_vec(message).unwrap_or_default();
    line.push(b'\n');
    line
}
