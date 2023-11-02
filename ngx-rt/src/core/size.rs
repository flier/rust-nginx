use std::fmt;

pub struct SizeFmt(pub usize);

impl fmt::Display for SizeFmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 >= MB && self.0 % MB == 0 {
            write!(f, "{}M", self.0 / MB)
        } else if self.0 >= KB && self.0 % KB == 0 {
            write!(f, "{}K", self.0 / KB)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

const KB: usize = 1024;
const MB: usize = 1024 * 1024;
