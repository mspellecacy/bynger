
pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
    let mut res: Vec<i32> = vec![];
    for (i1, i2) in nums.iter().zip(nums.iter()) {
        if (i1 != i2) && (nums[i1] + nums[i2]) {
            println!("{:#?}", nums[i1 as usize] + nums[i2 as usize]);
        }
    }
    res
}

fn main () {
    let nums: Vec<i32> = vec![2,7,11,15];
    let target = 9;

    assert_eq!(two_sum(nums, target), [0,1]);

    assert_eq!(two_sum(vec![3,2,3], 6), [0, 2]);
}