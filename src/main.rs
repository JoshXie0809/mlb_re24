use std::mem;
use rand::{distributions::WeightedIndex, prelude::Distribution, rngs::ThreadRng, thread_rng};
use rayon::prelude::*;

// possible hit result
const HITS: [HitResult; 9] = [
        HitResult::AO,
        HitResult::GO,
        HitResult::SO,
        HitResult::BB,
        HitResult::IBB,
        HitResult::OneBase,
        HitResult::TwoBase,
        HitResult::ThreeBase,
        HitResult::HR,
    ];

// expected run 
const ER: [[f64; 3]; 8] = [
    // _ _ _
    [0.4886, 0.2630, 0.1008],
    // 1 _ _
    [0.8577, 0.5115, 0.2213],
    // _ 2 _
    [1.0732, 0.6551, 0.3187],
    // 1 2 _
    [1.4423, 0.9036, 0.4392],
    // _ _ 3
    [1.3081, 0.8977, 0.3634],
    // 1 _ 3
    [1.6772, 1.1462, 0.4839],
    // _ 2 3
    [1.8927, 1.2898, 0.5813],
    // 1 2 3
    [2.2618, 1.5383, 0.7018],
];

fn main() -> Result<(), csv::Error>{
    let otani = Player::new(
        511, 
        150, 
        29, 
        6, 
        41, 
        69, 
        8, 
        0.78, 
        130
    );

    let res = test1(&otani, 100_000_000);
    let csv_file = "ohtani_expected_run.csv";
    let mut wtr = csv::Writer::from_path(csv_file)?;
    for row in res.0 {
        wtr.serialize(row)?;
    }
    wtr.flush()?;


    let csv_file = "ohtani_variance_of_run.csv";
    let mut wtr = csv::Writer::from_path(csv_file)?;
    for row in res.1 {
        wtr.serialize(row)?;
    }
    wtr.flush()?;

    Ok(())
}


fn test1(player: &Player, n_iter: i32) -> ([[f64; 3]; 8], [[f64; 3]; 8]) {
    let mut player_er    : [[f64; 3]; 8] = ER.clone();
    let mut player_er_std: [[f64; 3]; 8] = ER.clone();

    for one_base in [false, true] {
        for two_base in [false, true] {
            for three_base in [false, true] {
                for out in [0, 1, 2] {

                    let r = 
                        1 * one_base as i32 + 
                        2 * two_base as i32  + 
                        4 * three_base as i32;

                    let c = out;
                    let (r, c) = (r as usize, c as usize);

                    let weights: [i32; 9] = player.stat();
                    let dist = WeightedIndex::new(weights).unwrap();
                    
                    let res = (1..n_iter).into_par_iter().map(|_| {
                        let mut rng = thread_rng();
                        let (er24, _) = one_batting(&dist, &mut rng, one_base, two_base, three_base, out);
                        er24
                    }).collect::<Vec<f64>>();

                    let mu = res.iter().sum::<f64>() / n_iter as f64;
                    // standard deviation^2
                    let variance = res.iter().map(|val| {
                        val * val
                    }).sum::<f64>() / (n_iter as f64 - 1.0) - mu * mu;

                    player_er[r][c] = mu;
                    player_er_std[r][c] = variance;
                }
            }
        }
    }

    (player_er, player_er_std)
}


fn one_batting(
        dist:&WeightedIndex<i32>,
        rng: &mut ThreadRng,
        one_base: bool,
        two_base: bool,
        three_base: bool,
        out: i32,
    ) -> (f64, bool)
{
    let mut situation = BaseSituation::new();
    situation.one_base = one_base;
    situation.two_base = two_base;
    situation.three_base = three_base;
    situation.out = out;
    
    let onep = &HITS[dist.sample(rng)];
    situation.update(onep)
}



#[derive(Debug)]
struct Player {
    stat: [i32; 9]
}


impl Player {
    fn new( ab: i32, h: i32, 
            two_b: i32, three_b: i32, hr: i32,
            bb: i32, ibb: i32, goao: f64, so: i32,
        ) -> Self 
    {
        let mut stat: [i32; 9] = [0; 9];

        // bip
        let bip: i32 = ab - h - so;
        let go_ratio: f64 = goao / (1.0 + goao);
        let go: i32 = (bip as f64 * go_ratio).round() as i32;
        let ao: i32 = bip - go;

        // compute 1b
        let one_b: i32 = h - two_b - three_b - hr;
        
        // 1. ao
        stat[0] = ao;
        // 2. go
        stat[1] = go;
        // 3. so
        stat[2] = so;
        // 4. bb
        stat[3] = bb;
        // 5. ibb
        stat[4] = ibb;
        // 6. 1b
        stat[5] = one_b;
        // 7. 2b
        stat[6] = two_b;
        // 8. 3b
        stat[7] = three_b;
        // 9. ht
        stat[8] = hr;

        Player {
            stat
        }
    }
    
    fn stat(&self) -> [i32; 9] {
        self.stat
    }
}


#[derive(Debug)]
pub enum HitResult {
    AO, 
    GO,
    SO,
    BB,
    IBB,
    OneBase,
    TwoBase,
    ThreeBase,
    HR,
}

#[derive(Debug)]
pub struct BaseSituation {
    pub one_base: bool,
    pub two_base: bool,
    pub three_base: bool,
    pub out: i32,
}

impl BaseSituation {
    fn new() -> Self {
        BaseSituation {
            one_base: false,
            two_base: false,
            three_base: false,
            out: 0,
        }
    }

    fn update(&mut self, one_possible: &HitResult) -> (f64, bool) {
        
        let (r, c) = self.calc_row_column();
        let init_er: f64 = ER[r][c];
        
        let run = match one_possible {
            HitResult::AO => {
                ao_updae(self)      
            }

            HitResult::GO | HitResult::SO => {
                go_so_undate(self)
            },

            HitResult::BB | HitResult::IBB => {
                bb_ibb_update(self)
            },

            HitResult::OneBase => {
                one_b_update(self)
            },

            HitResult::TwoBase => {
                two_b_update(self)
            }

            HitResult::ThreeBase => {
                three_b_update(self)
            }

            HitResult::HR => {
                hr_update(self)
            }
        };

        let (after_er, inning_finished) =  if self.out >= 3 {
            let _ = mem::replace(self, BaseSituation::new());
            (0.0, true)
        } else {
            let (r, c) = self.calc_row_column();
            (ER[r][c], false)
        };

        let delta_er = after_er - init_er;
        (run as f64 + delta_er, inning_finished)

    }

    fn calc_row_column(&self) -> (usize, usize) {
        let r = 
            1 * self.one_base as i32 + 
            2 * self.two_base as i32  + 
            4 * self.three_base as i32;

        let c = self.out;

        (r as usize, c as usize)
    }
}


fn ao_updae(situation: &mut BaseSituation) -> f64 {
    let mut run = 0;
    situation.out += 1;

    if situation.out >= 3 {
        return 0.0;
    }

    if situation.three_base {
        situation.three_base = false;
        run += 1;
    }
    
    if situation.two_base {
        situation.two_base = false;
        situation.three_base = true;
    }

    if situation.one_base {
        situation.one_base = false;
        situation.two_base = true;
    }

    run as f64
}

fn go_so_undate(situation: &mut BaseSituation) -> f64 {
    situation.out += 1;
    0.0
}

fn bb_ibb_update(situation: &mut BaseSituation) -> f64 {
    let mut run = 0;

    if !situation.one_base {
        situation.two_base = true;
    } else {
        if !situation.two_base {
            situation.two_base = true;
        } else {
            if !situation.three_base {
                situation.two_base = true;
            } else {
                run += 1;
            }
        }
    }

    run as f64
}

fn one_b_update(situation: &mut BaseSituation) -> f64 {
    let mut run = 0;
    if situation.three_base {
        situation.three_base = false;
        run += 1;
    }
    
    if situation.two_base {
        situation.two_base = false;
        situation.three_base = true;
    }

    if situation.one_base {
        situation.one_base = false;
        situation.two_base = true;
    }

    situation.one_base = true;

    run as f64
}

fn two_b_update(situation: &mut BaseSituation) -> f64 {
    let mut run = 0;
    if situation.three_base {
        situation.three_base = false;
        run += 1;
    }
    
    if situation.two_base {
        situation.two_base = false;
        run += 1;
    }

    if situation.one_base {
        situation.one_base = false;
        situation.three_base = true;
    }

    situation.two_base = true;
    run as f64
}

fn three_b_update(situation: &mut BaseSituation) -> f64 {
    let mut run = 0;
    if situation.three_base {
        situation.three_base = false;
        run += 1;
    }
    
    if situation.two_base {
        situation.two_base = false;
        run += 1;
    }

    if situation.one_base {
        situation.one_base = false;
        run += 1;
    }

    situation.three_base = true;
    run as f64
}

fn hr_update(situation: &mut BaseSituation) -> f64 {
    let mut run = 0;
    if situation.three_base {
        situation.three_base = false;
        run += 1;
    }
    
    if situation.two_base {
        situation.two_base = false;
        run += 1;
    }

    if situation.one_base {
        situation.one_base = false;
        run += 1;
    }

    run += 1;
    run as f64
}