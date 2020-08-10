use super::timer;

pub struct RaceManager {
    last_checkpoint : u8,
    last_checkpoint_time : f32,
    first_checkpoint_time : f32,
}

impl RaceManager {
    pub fn new() -> Self{
        Self {
            last_checkpoint : 255, // no last checkpoint
            last_checkpoint_time : timer.elapsed().as_secs_f32(),
            first_checkpoint_time : 0.0,
        }
    }
    pub fn eval(&mut self, checkpoint : u8) -> bool {
        if self.last_checkpoint == 255 {
            self.first_checkpoint_time = timer.elapsed().as_secs_f32();
        }

        if  ( timer.elapsed().as_secs_f32() - self.last_checkpoint_time )  <= 10.0 || self.last_checkpoint == 255 {
            if checkpoint == self.last_checkpoint+1 {
                self.last_checkpoint = checkpoint;
                self.last_checkpoint_time = timer.elapsed().as_secs_f32();
                return true
            } else if checkpoint == self.last_checkpoint {
                return false
            }  else {
                self.last_checkpoint = 255;
                return false;
            }
        } else {
            self.last_checkpoint = 255;
            return false;
        }
    }

    pub fn reset(&mut self) {
        self.last_checkpoint = 255;
    }

    pub fn get_total_time(&self) -> f32 {
        timer.elapsed().as_secs_f32() - self.first_checkpoint_time
    }
}