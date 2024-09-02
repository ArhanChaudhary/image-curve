use num::{Integer, Num, NumCast};

pub fn lerp<T: Integer + NumCast + Copy, const N: usize, R: Num + NumCast>(
    values: [T; N],
    percentage: usize,
) -> R {
    let percentage_jump = 100.0 / (N as f64 - 1.0);
    let floored_index = (percentage as f64 / percentage_jump) as usize;
    if floored_index == values.len() - 1 {
        return num::cast(values[N - 1]).unwrap();
    }
    let floored_val = values[floored_index].to_f64().unwrap();
    let ceiled_val = values[floored_index + 1].to_f64().unwrap();
    let lerp_percentage =
        (percentage as f64 - percentage_jump * floored_index as f64) / percentage_jump;
    let lerp = ((ceiled_val - floored_val) * lerp_percentage + floored_val).round();
    num::cast(lerp).unwrap()
}
