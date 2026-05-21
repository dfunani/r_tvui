pub fn decrement(counter: &mut u32, amount: u32) {
    if *counter == 0 {
        return;
    }
    if *counter - amount >= 0 {
        *counter -= amount;
    }
}

pub fn increment(counter: &mut u32, amount: u32) {
    *counter += amount;
}
