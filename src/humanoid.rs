use rand::Rng;

#[derive(PartialEq, Clone, Debug)]
pub enum Mood {
    Happy = 6,
    Glad = 5,
    Okay = 4,
    Sad = 3,
    Aggressive = 2,
    Depressive = 1,
    Deactivated = 0
}

impl std::fmt::Display for Mood {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let smiley = match self {
            Mood::Happy => "=D",
            Mood::Glad => "=)",
            Mood::Okay => "=I",
            Mood::Sad => "=(",
            Mood::Aggressive => "=X",
            Mood::Depressive => "X/",
            Mood::Deactivated => "Xc"
        };
        write!(formatter, "{}", smiley)
    }
}

pub trait Humanoid {
    fn call(&mut self);
    fn mood_level(&self) -> u32;
    fn mood_changed(&mut self) -> bool;
    fn mood_range(&self) -> [u32; 6];
}

pub struct Worker {
    prev_mood: Mood,
    stress_level: u32
}

pub struct Shouter {
    voice_damage: u32,
    robot: bool
}

impl Worker {
    pub fn new() -> Self {
        Worker {
            prev_mood: Mood::Happy,
            stress_level: 0
        }
    }
}

impl Shouter {
    pub fn new() -> Self {
        Shouter {
            voice_damage: 0,
            robot: false
        }
    }

    pub fn shout(&mut self, shout_level: usize, text: String) {
        if self.robot  {
            println!("{}", text);
        } else {
        let mut rng = rand::thread_rng();
        if self.voice_damage > 1000 {
            println!("{} {}", HumanoidControl::mood::<Shouter>(&self), 
            match rng.gen_range(1..4) {
                1 => {
                    "*hust*"
                }, 
                2 => {"*keuch*"},
                3 => {"*arr*"},
                _ => {"*hrrm*"}
            });
        } else {
            let mut s = String::new();
            for c in text.chars() {
                let r = rng.gen_range(0..100);
                if ((shout_level-1)*10) > r {
                    for upper_c in c.to_uppercase() {
                        s.push(upper_c);
                    }
                } else {
                    s.push(c);
                }
            }
            println!("{}", s);
            self.voice_damage = self.voice_damage + shout_level as u32;
        }
    }
}
}

pub struct HumanoidControl {}

impl HumanoidControl{
    pub fn mood<T: Humanoid>(humanoid: &T) -> Mood {
        let n = humanoid.mood_level();
        let range = humanoid.mood_range();
        if n < range[0] {
            Mood::Happy
        } else if n < range[1] {
            Mood::Glad
        } else if n < range[2] {
            Mood::Okay
        } else if n < range[3] {
            Mood::Sad
        } else if n < range[4] {
            Mood::Aggressive
        } else if n < range[5] {
            Mood::Depressive
        } else {
            Mood::Deactivated
        }
    }
}

impl Humanoid for Worker {
    fn call(&mut self) {
        self.stress_level = self.stress_level + 1;
    }

    fn mood_range(&self) -> [u32; 6] {
        [20, 30, 40, 100, 1000, 10000]
    }

    fn mood_level(&self) -> u32 {
        self.stress_level
    }

    fn mood_changed(&mut self) -> bool {
        let last = self.prev_mood.clone();
        let new_mood = HumanoidControl::mood::<Worker>(&self);
        let result = last != new_mood;
        self.prev_mood = new_mood;
        return result
    } 
}

impl Humanoid for Shouter {
    fn call(&mut self) {
        self.voice_damage = self.voice_damage + 1;
    }

    fn mood_range(&self) -> [u32; 6] {
        [20, 30, 40, 100, 1000, 10000]
    }

    fn mood_level(&self) -> u32 {
        self.voice_damage
    }

    fn mood_changed(&mut self) -> bool {
        return false
    } 
}