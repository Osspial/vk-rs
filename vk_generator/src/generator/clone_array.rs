{0}: {{
    use std::mem;
    let mut array: [_; {1}] = unsafe{{ mem::uninitialized() }};

    array.clone_from_slice(&self.{0});

    array
}},
