pub struct AntiSpam {
    packets_per_second : u16,
    last_time : f32,
}

impl AntiSpam {
    pub fn new() -> Self {
        Self {
            packets_per_second : 0,
            last_time : 0.0,
        }
    }
    pub fn tick_and_is_ok(&mut self) -> bool {
        if super::timer.elapsed().as_secs_f32()>self.last_time+1.0 {
            self.last_time+=1.0;
            self.packets_per_second = 0;
        }
        self.packets_per_second+=1;
        self.packets_per_second < *super::packets_per_second_limit
    }
}