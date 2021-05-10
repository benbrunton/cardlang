use crate::cards::Card;
use rand::seq::SliceRandom;

pub fn shuffle(stack: &mut Vec<Card>) {
    let mut rng = rand::thread_rng();
    stack.shuffle(&mut rng);
}