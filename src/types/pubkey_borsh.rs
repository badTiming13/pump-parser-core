use borsh::BorshDeserialize;
use bs58;
use std::fmt;

#[derive(BorshDeserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pubkey(pub [u8; 32]);

impl fmt::Debug for Pubkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = bs58::encode(self.0).into_string();
        write!(f, "{s}")
    }
}

impl fmt::Display for Pubkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = bs58::encode(self.0).into_string();
        write!(f, "{s}")
    }
}
