use super::*;

mod acmd;
mod status;

pub fn install() {
    let agent = &mut Agent::new("koopa_breath");
    agent.set_costume(vec![55]);
    acmd::install(agent);
    status::install(agent);
    agent.install();
}