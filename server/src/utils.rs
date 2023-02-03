use ethers::types::U256;

const M: f64 = 100_f64;
const S: i32 = 10;

// R= B * ((P*10^4) / (10^S +(P*10^4) )) * M/100
pub fn get_share(points: f64) -> f64 {
    (points * 10_f64.powi(4)) / (10_f64.powi(S) + (points * 10_f64.powi(4)))
}

pub fn get_reward(total: f64, share: f64) -> f64 {
    total * share * M / 100_f64
}

pub fn uint_to_float(value: U256, decimals: u8) -> f64 {
    let value = value.as_u128() as f64;
    let div = 10_u128.pow(decimals as u32) as f64;

    value / div
}
