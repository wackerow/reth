use core::ops::Deref;

use crate::Compact;
use alloy_eips::eip7702::{Authorization as AlloyAuthorization, SignedAuthorization};
use alloy_primitives::{Address, ChainId, U256};
use bytes::Buf;
use reth_codecs_derive::main_codec;

/// Authorization acts as bridge which simplifies Compact implementation for AlloyAuthorization.
///
/// Notice: Make sure this struct is 1:1 with `alloy_eips::eip7702::Authorization`
#[main_codec]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Authorization {
    chain_id: ChainId,
    address: Address,
    nonce: Option<u64>,
}

impl Compact for AlloyAuthorization {
    fn to_compact<B>(self, buf: &mut B) -> usize
    where
        B: bytes::BufMut + AsMut<[u8]>,
    {
        let authorization =
            Authorization { chain_id: self.chain_id, address: self.address, nonce: self.nonce() };
        authorization.to_compact(buf)
    }

    fn from_compact(buf: &[u8], len: usize) -> (Self, &[u8]) {
        let (authorization, _) = Authorization::from_compact(buf, len);
        let alloy_authorization = AlloyAuthorization {
            chain_id: authorization.chain_id,
            address: authorization.address,
            nonce: authorization.nonce.into(),
        };
        (alloy_authorization, buf)
    }
}

impl Compact for SignedAuthorization<alloy_primitives::Signature> {
    fn to_compact<B>(self, buf: &mut B) -> usize
    where
        B: bytes::BufMut + AsMut<[u8]>,
    {
        let mut buffer = Vec::new();
        // todo: add `SignedAuthorization::into_parts(self) -> (Auth, Signature)`
        let (auth, signature) = (self.deref().clone(), self.signature());
        let (v, r, s) = (signature.v(), signature.r(), signature.s());
        auth.to_compact(&mut buffer);
        buf.put_u8(v.y_parity_byte());
        buf.put_slice(r.as_le_slice());
        buf.put_slice(s.as_le_slice());

        let total_len = buffer.len();
        buf.put(buffer.as_slice());
        total_len
    }

    fn from_compact(buf: &[u8], len: usize) -> (Self, &[u8]) {
        let (auth, mut buf) = AlloyAuthorization::from_compact(buf, len);
        let y = buf.get_u8() == 1;
        let r = U256::from_le_slice(&buf[0..32]);
        buf.advance(32);
        let s = U256::from_le_slice(&buf[0..32]);
        buf.advance(32);

        let signature = alloy_primitives::Signature::from_rs_and_parity(r, s, y)
            .expect("invalid authorization signature");
        (auth.into_signed(signature), buf)
    }
}

// TODO(eip7702): complete these tests
#[cfg(test)]
mod tests {
    /*
    use super::*;
    use alloy_primitives::{address, b256};

    #[test]
    fn test_roundtrip_compact_authorization_list_item() {
        let authorization = Authorization {
            chain_id: 1,
            address: address!("dac17f958d2ee523a2206206994597c13d831ec7"),
            nonce: None,
            y_parity: false,
            r: b256!("1fd474b1f9404c0c5df43b7620119ffbc3a1c3f942c73b6e14e9f55255ed9b1d").into(),
            s: b256!("29aca24813279a901ec13b5f7bb53385fa1fc627b946592221417ff74a49600d").into(),
        };
        let mut compacted_authorization = Vec::<u8>::new();
        let len = authorization.clone().to_compact(&mut compacted_authorization);
        let (decoded_authorization, _) = Authorization::from_compact(&compacted_authorization, len);
        assert_eq!(authorization, decoded_authorization);
        }*/
}
