use commonware_cryptography::{ed25519, Digest};
use commonware_utils::bitmap::BitMap;
use crate::types::{Round, View};

#[derive(Clone, Debug, PartialEq)]
pub struct Vote {
    /// Index of the signer inside the participant set.
    pub signer: u32,
    /// Signature of the vote.
    pub signature: ed25519::Signature,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Proposal<D: Digest> {
    /// The round in which this proposal is made
    pub round: Round,
    /// The view of the parent proposal that this one builds upon
    pub parent: View,
    /// The actual payload/content of the proposal (typically a digest of the block data)
    pub payload: D,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Certificate {
    pub signers: BitMap<1>,
    pub signatures: Vec<ed25519::Signature>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Notarize<D: Digest> {
    /// Proposal being notarized.
    pub proposal: Proposal<D>,
    /// Vote material
    pub vote: Vote,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Notarization<D: Digest> {
    /// Proposal being notarized.
    pub proposal: Proposal<D>,
    /// Certificate
    pub certificate: Certificate,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Nullify {
    pub round: Round,
    pub vote: Vote,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Nullification {
    pub round: Round,
    pub certificate: Certificate,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Voter<D: Digest> {
    Notarize(Notarize<D>),
    Notarization(Notarization<D>),
    Nullify(Nullify),
    Nullification(Nullification),
}