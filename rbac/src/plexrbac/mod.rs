mod domain;
mod security;
mod persistence;
mod utils;
mod common;

pub fn top() {
    let factory = persistence::factory::RepositoryFactory::new();
    let repo = factory.new_realm_repository();
    repo.clear();
}
