use crate::hrs3300::RawSample as Rs;

#[allow(unused)]
pub const VALUES_BUFFER_LENGTH: usize = 100;

#[allow(unused)]
pub struct PpgFilter {
    pub values_buffer: [Rs; VALUES_BUFFER_LENGTH],  
    cursor: usize,  
}
impl PpgFilter{
    #[allow(unused)]
    pub fn new () -> Self {
        PpgFilter {
            values_buffer: [
                Rs::new(0_u32, 0_u32); 
                VALUES_BUFFER_LENGTH
            ],
            cursor: 0
        }
    }

    #[allow(unused)]
    pub fn consume_value(&mut self, value: Rs) -> i64 {
        self.cursor += 1;
        return if self.cursor >= self.values_buffer.len() {
            self.cursor = self.values_buffer.len();
            self.values_buffer.rotate_left(1);
            *(self.values_buffer.last_mut().unwrap()) = value;
            value.get_sum() as i64 - Self::get_avg(&self) as i64
        } else {
            0_i64
        }        
    }

    fn get_avg(&self) -> i64 {
        let mut avg = 0_i64;
        for value in self.values_buffer.iter() {
             avg += value.get_sum() as i64;
        }
        avg / self.values_buffer.len() as i64
    }
}