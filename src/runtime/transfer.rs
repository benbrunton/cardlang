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
    mut to: Option<TransferTarget>,
    mut from: Option<TransferTarget>,
    t_count: Option<&TransferCount>
) -> Option<(TransferTarget, TransferTarget)> {
    let mut count = match t_count {
        None => 1,
        Some(TransferCount::End) => from.as_ref().unwrap().count()
    };

    // multiply by number of target stacks
    count *= match &to {
        Some(TransferTarget::Stack(_)) => 1,
        Some(TransferTarget::StackList(s)) => s.len(),
        _ => 0
    };

    let mut transfer_index = 0;

    while count > 0 {

        let card_result = match from {
            Some(TransferTarget::Stack(ref mut s)) => s.pop(),
            _ => None
        };

        // todo - error?
        if card_result.is_none() {
            break;
        }

        if to.is_none() {
            return None;
        }

        let card = card_result.expect("unable to get card");

        match to {
            Some(TransferTarget::StackList(ref mut s)) => {
                s[transfer_index].push(card);
                if transfer_index >= s.len() - 1 {
                    transfer_index = 0;
                } else {
                    transfer_index += 1
                }
            },
            _ => ()
        }
        count -= 1;
    }

    Some((from.unwrap(), to.unwrap()))
}