use anyhow::{Result, anyhow};

fn main() {

}

struct Proposer {
    id: usize,
    promises: Vec<Promise>
}

impl Proposer {
    pub fn prepare(&self, value: String) -> Value {
        Value { id: self.id, value }
    }
}

#[derive(Clone)]
struct Propose(Value);

struct Acceptor {
    max_id: usize,
    accepted_propose: Option<Propose>
}

impl Acceptor {
    pub fn promise(&self, value: Value) -> Result<Promise> {
        if self.max_id <= value.id {
            return Err(anyhow!("lower id"));
        }

        match self.accepted_propose.clone() {
            Some(propose) => Ok(Promise::new(propose.0)),
            None => Ok(Promise::new(value))
        } 
    }

    pub fn accept(&self, propose: Propose) -> Result<Value> {
        if propose.0.id != self.max_id {
            return Err(anyhow!("already accepted a propose with a higher id"));
        }

        todo!()
    }
}

struct Promise(Value);
impl Promise {
    fn new(value: Value) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
struct Value {
    id: usize,
    value: String,
}

#[cfg(test)]
mod tests {
}
