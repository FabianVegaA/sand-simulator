#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Dead,
    Alive,
    MouseOver(bool), // bool is true if the cell is alive
    MouseOut,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Sand,
    Rock,
}

#[derive(Clone, Copy)]
pub struct Cellule {
    pub kind: Option<Kind>,
    pub state: State,
    pub pressure: u8,
}

impl Cellule {
    pub fn set_kind(&mut self, kind: Kind) -> &mut Self {
        self.kind = Some(kind);
        self
    }

    pub fn set_pressure(&mut self, pressure: u8) -> &mut Self {
        self.pressure = pressure;
        self
    }

    pub fn is_alive(&mut self) -> bool {
        self.state == State::Alive || self.state == State::MouseOver(true)
    }

    pub fn is_dead(&mut self) -> bool {
        !self.is_alive()
    }

    pub fn set_alive(&mut self) {
        self.state = State::Alive;
    }

    pub fn set_dead(&mut self) {
        self.state = State::Dead;
        self.kind = None;
        self.pressure = 0;
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
