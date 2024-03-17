#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Dead,
    Alive,
    MouseOver(bool), // bool is true if the cell is alive
    MouseOut,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Sand,
    Rock,
}

#[derive(Clone, Copy)]
pub struct Cellule {
    pub kind: Kind,
    pub state: State,
}

impl Cellule {
    pub fn update_kind(&mut self, kind: Kind) {
        self.kind = kind;
    }

    pub fn is_alive(&mut self) -> bool {
        self.state == State::Alive || self.state == State::MouseOver(true)
    }

    pub fn is_dead(&mut self) -> bool {
        !self.is_alive()
    }

    pub fn swap(&mut self) {
        if self.is_alive() {
            self.set_dead();
        } else {
            self.set_alive();
        }
    }

    pub fn set_alive(&mut self) {
        self.state = State::Alive;
    }

    pub fn set_dead(&mut self) {
        self.state = State::Dead;
    }

    pub fn set_mouse_over(&mut self) {
        self.state = State::MouseOver(self.is_alive());
    }

    pub fn set_mouse_out(&mut self) {
        self.state = match self.state {
            State::MouseOver(true) | State::Alive => State::Alive,
            _ => State::MouseOut,
        }
    }
}
