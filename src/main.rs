use rand::{distributions::WeightedIndex, prelude::Distribution, rngs::ThreadRng, thread_rng};

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




fn main() {
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

    let choice = simu_1(&otani);
    println!("{:?}", choice);
}


fn simu_1 (player: &Player) -> &HitResult {
    
    let weights: [i32; 9] = player.stat();
    let dist = WeightedIndex::new(weights).unwrap();
    let mut rng = thread_rng();
    let choice = hit(&HITS, &dist, &mut rng);
    choice
}


fn hit<'a> (hit_result: &'a [HitResult; 9], dist:&WeightedIndex<i32>, rng: &mut ThreadRng) -> &'a HitResult {
    &hit_result[dist.sample(rng)]
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