//use crate::ast::*;
use crate::cards::Card;
use rand::seq::SliceRandom;


pub fn inbuilt(name: &str, mut args: Vec<&mut Vec<Card>>) -> Option<()> {
    shuffle(args[0]);
    Some(())
}

fn shuffle(stack: &mut Vec<Card>) {
    let mut rng = rand::thread_rng();
    stack.shuffle(&mut rng);
}