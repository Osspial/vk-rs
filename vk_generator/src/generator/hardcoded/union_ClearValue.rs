/// Temporary Hard-Coded union hack; will be automatically generated when actual unions become stable
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct {0} {{
    data: [u8; 16]
}}

impl {0} {{
    pub unsafe fn color(&self) -> &{1} {{
        use std::mem;
        mem::transmute(&self.data)
    }}

    pub unsafe fn depth_stencil(&self) -> &{2} {{
        use std::mem;
        mem::transmute(&self.data)
    }}

    pub unsafe fn color_mut(&mut self) -> &mut {1} {{
        use std::mem;
        mem::transmute(&mut self.data)
    }}

    pub unsafe fn depth_stencil_mut(&mut self) -> &mut {2} {{
        use std::mem;
        mem::transmute(&mut self.data)
    }}

    pub fn new_color(color: {1}) -> {0} {{
        use std::mem;
        unsafe {{
            let mut union: {0} = mem::zeroed();
            union.data = mem::transmute(color);
            union
        }}
    }}

    pub fn new_depth_stencil(depth_stencil: {2}) -> {0} {{
        use std::mem;
        unsafe {{
            let mut union: {0} = mem::zeroed();
            union.data = mem::transmute_copy(&depth_stencil);
            union
        }}
    }}
}}
