use crate::game::state::State;

pub struct FinishState {}
#[async_trait::async_trait]
impl State for FinishState {}
