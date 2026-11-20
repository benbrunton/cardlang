use crate::cards::Card;
use crate::ast::*;

pub enum TransferTarget {
    Stack(Stack),
    StackList(Vec<Stack>)
}

impl TransferTarget {
    pub fn count(&self) -> usize {
        match self {
            TransferTarget::Stack(s) => s.len(),
            TransferTarget::StackList(s) => s.len()
        }
    }

    pub fn get_stack(&self, n: usize) -> Stack {
        match self {
            TransferTarget::Stack(s) => s.clone(),
            TransferTarget::StackList(s) => s[n].clone()
        }
    }
}

pub type Stack = Vec<Card>;

pub fn transfer(
    mut from: Option<TransferTarget>,
    mut to: Option<TransferTarget>,
    t_count: Option<&TransferCount>
) -> Option<(TransferTarget, TransferTarget)> {
    let mut count = match t_count {
        None => 1,
        Some(TransferCount::End) => from.as_ref().map_or(0, |f| f.count()),
        Some(TransferCount::Number(n)) => *n,
        // specific cards are moved by the runtime, not here
        Some(TransferCount::Taking(_)) => return None
    };

    // counts are per destination stack, so multiply by the number of them
    // (for `end` this is harmless: the loop stops when the source is empty)
    count *= match &to {
        Some(TransferTarget::Stack(_)) => 1,
        Some(TransferTarget::StackList(s)) => s.len(),
        _ => 0
    };

    let mut transfer_index = 0;

    while count > 0 {

        // nothing to transfer into
        to.as_ref()?;

        // running out of cards early is fine: move as many as there are
        let card = match from {
            Some(TransferTarget::Stack(ref mut s)) => match s.pop() {
                Some(card) => card,
                None => break
            },
            _ => break
        };

        match to {
            Some(TransferTarget::StackList(ref mut s)) => {
                s[transfer_index].push(card);
                if transfer_index >= s.len() - 1 {
                    transfer_index = 0;
                } else {
                    transfer_index += 1
                }
            },
            Some(TransferTarget::Stack(ref mut s)) => s.push(card),
            _ => ()
        }
        count -= 1;
    }

    match from {
        Some(f) => to.map(|t| (f, t)),
        _ => None
    }
}

#[cfg(test)]
mod test{
    use super::*;
    use crate::cards::standard_deck;

    #[test]
    fn it_can_handle_moving_stack_to_stack() {
        let from = Some(TransferTarget::Stack(standard_deck()));
        let to = Some(TransferTarget::Stack(vec![]));

        let result = transfer(from, to, None);

        let (_new_from, new_to) = result.unwrap();

        assert_eq!(new_to.count(), 1);
    }
}