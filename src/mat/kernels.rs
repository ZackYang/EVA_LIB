#[derive(Debug)]
pub struct Kernel {
    pub data: Vec<Vec<f32>>,
}

impl Kernel {
    pub fn load(data: Vec<Vec<f32>>)
        -> Kernel
    {
        let kernel = Kernel { data: data };
        kernel
    }

    pub fn laplation_8() -> Kernel {
        Kernel::load(vec![
            vec![1.0, 1.0, 1.0],
            vec![1.0, -8.0, 1.0],
            vec![1.0, 1.0, 1.0]
        ])
    }

    pub fn laplation_4() -> Kernel {
        Kernel::load(vec![
            vec![0.0, 1.0, 0.0],
            vec![1.0, -4.0, 1.0],
            vec![0.0, 1.0, 0.0]
        ])
    }

    pub fn laplation_12() -> Kernel {
        Kernel::load(vec![
            vec![1.0, 1.0, 1.0, 1.0],
            vec![1.0, -3.0, -3.0, 1.0],
            vec![1.0, -3.0, -3.0, 1.0],
            vec![1.0, 1.0,   1.0, 1.0]
        ])
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn flatten(&self)
        -> Vec<f32> 
    {
        let mut vec = Vec::with_capacity(self.elements());
        for row in self.data.to_vec() {
            for value in row {
                vec.push(value);
            }
        }
        vec
    }

    pub fn elements(&self)
        -> usize
    {
        self.data.len()*self.data.len()
    }

    pub fn indexes(&self, index: usize, chrunk_size: usize, total_elements: usize)
        -> (bool, Vec<usize>)
    {
        let mut indexes = Vec::with_capacity(self.size()*self.size());
        let offset = index%chrunk_size;

        if (offset + self.size()) > chrunk_size {
            return (false, vec![]);
        }

        for row in 0..self.size() {
            for col in 0..self.size() {
                let id = row * chrunk_size + col + index;
                if id >= total_elements {
                    return (false, vec![]);
                }
                indexes.push(id);
            }   
        }
        return (true, indexes);
    }

}