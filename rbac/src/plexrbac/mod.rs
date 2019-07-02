mod domain;
mod security;
mod persistence;
mod utils;

pub fn top() {
    let factory = persistence::factory::RepositoryFactory::new();
    let repo = factory.new_realm_repository();
    repo.clear();
}
