use pasetors::{
    version4::V4,
    keys::{SymmetricKey, Generate},
};
use anyhow::Result;

#[derive(Clone)]
pub struct PasetoSecurityAdapter {
    pub sk: SymmetricKey<V4>,
}

impl PasetoSecurityAdapter {
    pub fn new() -> Result<Self> {
        let sk = SymmetricKey::<V4>::generate()?;

        Ok(Self { sk })
    }
}