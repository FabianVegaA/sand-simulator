#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Dead,
    Alive,
    MouseOver,
    MouseOut,
}

#[derive(Clone, Copy)]
pub struct Cellule {
    pub state: State,
}

impl Cellule {
    pub fn is_alive(&mut self) -> bool {
        self.state == State::Alive
    }

    pub fn is_dead(&mut self) -> bool {
        !self.is_alive()
    }

    pub fn swap(&mut self) {
        self.state = match self.state {
            State::Alive => State::Dead,
            State::Dead => State::Alive,
            state => state,
        }
    }

    pub fn set_alive(&mut self) {
        self.state = State::Alive;
    }

    pub fn set_dead(&mut self) {
        self.state = State::Dead;
    }

    pub fn set_mouse_over(&mut self) {
        self.state = State::MouseOver;
    }

    pub fn set_mouse_out(&mut self) {
        self.state = State::MouseOut;
    }
}
