#[derive(Debug)]
pub struct Kernel {
    pub data: Vec<Vec<f32>>
}

impl Kernel {
    pub fn load(data: Vec<Vec<f32>>)
        -> Kernel
    {
        Kernel { data: data }
    }

    pub fn new() -> Kernel {
        Kernel { data: vec![vec![0f32]] }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}