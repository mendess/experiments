use std::collections::TryReserveError;

trait VecExt<V> {
    fn push_checked(&mut self, v: V) -> Result<(), TryReserveError>;
}

impl<V> VecExt<V> for Vec<V> {
    fn push_checked(&mut self, v: V) -> Result<(), TryReserveError> {
        let before = self.capacity();
        if let Err(_) = self.try_reserve(1) {
            let mut additional = self.len() >> 1;
            loop {
                match self.try_reserve_exact(additional) {
                    Ok(_) => break,
                    Err(e) => {
                        additional >>= 1;
                        if additional == 0 {
                            return Err(e);
                        }
                    }
                }
            }
            let after = self.capacity();
            println!("used fallback {before} => {after} (+{additional})");
        } else {
            let after = self.capacity();
            if before != after {
                println!("used normal {before} => {after}");
            }
        }
        Ok(self.push(v))
    }
}

fn main() {
    let mut v = Vec::new();
    let e = loop {
        if let Err(e) = v.push_checked(0u8) {
            break e;
        }
    };
    let len = v.len();
    drop(v);
    println!("we got to {len} before os kicked us out because {e}");
}
