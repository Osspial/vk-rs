/// Temporary Hard-Coded union hack; will be automatically generated when actual unions become stable
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct {0} {{
    data: [u8; 16]
}}

impl {0} {{
    pub unsafe fn float32(&self) -> &[c_float; 4] {{
        use std::mem;
        mem::transmute(&self.data)
    }}

    pub unsafe fn int32(&self) -> &[int32_t; 4] {{
        use std::mem;
        mem::transmute(&self.data)
    }}

    pub unsafe fn uint32(&self) -> &[uint32_t; 4] {{
        use std::mem;
        mem::transmute(&self.data)
    }}

    pub unsafe fn float32_mut(&mut self) -> &mut [c_float; 4] {{
        use std::mem;
        mem::transmute(&mut self.data)
    }}

    pub unsafe fn int32_mut(&mut self) -> &mut [int32_t; 4] {{
        use std::mem;
        mem::transmute(&mut self.data)
    }}

    pub unsafe fn uint32_mut(&mut self) -> &mut [uint32_t; 4] {{
        use std::mem;
        mem::transmute(&mut self.data)
    }}

    pub fn new_float32(float32: [c_float; 4]) -> {0} {{
        use std::mem;
        unsafe {{
            let mut union: {0} = mem::zeroed();
            union.data = mem::transmute(float32);
            union
        }}
    }}

    pub fn new_int32(int32: [int32_t; 4]) -> {0} {{
        use std::mem;
        unsafe {{
            let mut union: {0} = mem::zeroed();
            union.data = mem::transmute(int32);
            union
        }}
    }}

    pub fn new_uint32(uint32: [uint32_t; 4]) -> {0} {{
        use std::mem;
        unsafe {{
            let mut union: {0} = mem::zeroed();
            union.data = mem::transmute(uint32);
            union
        }}
    }}
}}
