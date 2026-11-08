use super::node::{BorrowState, EngineError, NodeId};
use super::widget::WidgetKind;

struct Slot {
    generation: u32,
    borrow: BorrowState,
    widget: Option<WidgetKind>,
}

/// Manages generational memory slots and runtime borrow checking (`&` vs `&mut`).
#[derive(Default)]
pub struct Arena {
    slots: Vec<Slot>,
    free_indices: Vec<usize>,
}

impl Arena {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_indices: Vec::new(),
        }
    }

    pub fn insert(&mut self, widget: WidgetKind) -> NodeId {
        if let Some(index) = self.free_indices.pop() {
            let slot = &mut self.slots[index];
            slot.borrow = BorrowState::Unborrowed;
            slot.widget = Some(widget);
            NodeId {
                index,
                generation: slot.generation,
            }
        } else {
            let index = self.slots.len();
            let generation = 1;
            self.slots.push(Slot {
                generation,
                borrow: BorrowState::Unborrowed,
                widget: Some(widget),
            });
            NodeId { index, generation }
        }
    }

    fn get_slot(&self, id: NodeId) -> Result<&Slot, EngineError> {
        let slot = self
            .slots
            .get(id.index)
            .ok_or(EngineError::InvalidNode(id))?;
        if slot.generation != id.generation || slot.widget.is_none() {
            return Err(EngineError::StaleGeneration {
                id,
                current_generation: slot.generation,
            });
        }
        Ok(slot)
    }

    fn get_slot_mut(&mut self, id: NodeId) -> Result<&mut Slot, EngineError> {
        let slot = self
            .slots
            .get_mut(id.index)
            .ok_or(EngineError::InvalidNode(id))?;
        if slot.generation != id.generation || slot.widget.is_none() {
            return Err(EngineError::StaleGeneration {
                id,
                current_generation: slot.generation,
            });
        }
        Ok(slot)
    }

    pub fn get(&self, id: NodeId) -> Result<&WidgetKind, EngineError> {
        let slot = self.get_slot(id)?;
        if slot.borrow == BorrowState::Exclusive {
            return Err(EngineError::BorrowConflict {
                id,
                current_state: slot.borrow,
            });
        }
        slot.widget.as_ref().ok_or(EngineError::InvalidNode(id))
    }

    pub fn with_exclusive_mut<F, R>(&mut self, id: NodeId, f: F) -> Result<R, EngineError>
    where
        F: FnOnce(&mut WidgetKind) -> Result<R, EngineError>,
    {
        self.borrow_exclusive(id)?;
        let result = {
            let slot = self.get_slot_mut(id)?;
            f(slot.widget.as_mut().unwrap())
        };
        self.release_exclusive(id)?;
        result
    }

    pub fn borrow_shared(&mut self, id: NodeId) -> Result<(), EngineError> {
        let slot = self.get_slot_mut(id)?;
        match slot.borrow {
            BorrowState::Unborrowed => {
                slot.borrow = BorrowState::Shared(1);
                Ok(())
            }
            BorrowState::Shared(count) => {
                slot.borrow = BorrowState::Shared(count + 1);
                Ok(())
            }
            BorrowState::Exclusive => Err(EngineError::BorrowConflict {
                id,
                current_state: slot.borrow,
            }),
        }
    }

    pub fn release_shared(&mut self, id: NodeId) -> Result<(), EngineError> {
        let slot = self.get_slot_mut(id)?;
        match slot.borrow {
            BorrowState::Shared(1) => {
                slot.borrow = BorrowState::Unborrowed;
                Ok(())
            }
            BorrowState::Shared(count) if count > 1 => {
                slot.borrow = BorrowState::Shared(count - 1);
                Ok(())
            }
            _ => Err(EngineError::BorrowConflict {
                id,
                current_state: slot.borrow,
            }),
        }
    }

    pub fn borrow_exclusive(&mut self, id: NodeId) -> Result<(), EngineError> {
        let slot = self.get_slot_mut(id)?;
        match slot.borrow {
            BorrowState::Unborrowed => {
                slot.borrow = BorrowState::Exclusive;
                Ok(())
            }
            state => Err(EngineError::BorrowConflict {
                id,
                current_state: state,
            }),
        }
    }

    pub fn release_exclusive(&mut self, id: NodeId) -> Result<(), EngineError> {
        let slot = self.get_slot_mut(id)?;
        match slot.borrow {
            BorrowState::Exclusive => {
                slot.borrow = BorrowState::Unborrowed;
                Ok(())
            }
            state => Err(EngineError::BorrowConflict {
                id,
                current_state: state,
            }),
        }
    }

    pub fn remove(&mut self, id: NodeId) -> Result<(), EngineError> {
        let slot = self.get_slot_mut(id)?;
        if slot.borrow != BorrowState::Unborrowed {
            return Err(EngineError::BorrowConflict {
                id,
                current_state: slot.borrow,
            });
        }
        slot.widget = None;
        slot.generation = slot.generation.saturating_add(1);
        self.free_indices.push(id.index);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_stale_generation_after_remove() {
        let mut arena = Arena::new();
        let first = arena.insert(WidgetKind::from_tag("gtk-button").unwrap());
        assert_eq!(first.generation, 1);

        arena.remove(first).unwrap();

        let reused = arena.insert(WidgetKind::from_tag("gtk-button").unwrap());
        assert_eq!(reused.index, first.index);
        assert_eq!(reused.generation, 2);

        // Accessing with the old generation must fail
        assert_eq!(
            arena.get(first).unwrap_err(),
            EngineError::StaleGeneration {
                id: first,
                current_generation: 2
            }
        );
    }

    #[test]
    fn prevents_destruction_while_borrowed() {
        let mut arena = Arena::new();
        let node = arena.insert(WidgetKind::from_tag("gtk-button").unwrap());

        arena.borrow_shared(node).unwrap();
        assert_eq!(
            arena.remove(node).unwrap_err(),
            EngineError::BorrowConflict {
                id: node,
                current_state: BorrowState::Shared(1)
            }
        );

        arena.release_shared(node).unwrap();
        assert!(arena.remove(node).is_ok());
    }
}
