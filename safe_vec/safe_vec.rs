use std::collections::TryReserveError;

trait VecExt<V> {
    fn push_checked(&mut self, v: V) -> Result<(), TryReserveError>;
}

impl<V> VecExt<V> for Vec<V> {
    fn push_checked(&mut self, v: V) -> Result<(), TryReserveError> {
        self.try_reserve(1)?;
        Ok(self.push(v))
    }
}

fn main() {
    let mut v = Vec::new();
    let e = loop {
        if let Err(e) = v.push_checked(0) {
            break e;
        }
    };
    let len = v.len();
    drop(v);
    println!("[rust] we got to {len} before the os kicked us out because {e}");
}
