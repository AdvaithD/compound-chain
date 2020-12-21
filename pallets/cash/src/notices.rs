use super::chains::{Chain, Ethereum};
use codec::{Decode, Encode};
use ethabi;
use num_traits::ToPrimitive;
use our_std::{vec::Vec, Deserialize, RuntimeDebug, Serialize};
use secp256k1;
use tiny_keccak::Hasher;

// XXX
pub type Message = Vec<u8>;
pub type Signature = Vec<u8>; // XXX bunch of Signature types now, rename to avoid confusion?
pub type Signatures = Vec<Signature>;

pub type EraId = u32;
pub type EraIndex = u32;
pub type NoticeId = (EraId, EraIndex); // XXX make totally ordered trait

#[derive(Clone, Eq, PartialEq, Encode, Decode, Serialize, Deserialize, RuntimeDebug)]
pub enum Notice<C: Chain> {
    ExtractionNotice {
        id: NoticeId,
        parent: C::Hash,
        asset: C::Asset,
        account: C::Account,
        amount: C::Amount,
    },

    CashExtractionNotice {
        id: NoticeId,
        parent: C::Hash,
        account: C::Account,
        amount: C::Amount,
        cash_yield_index: C::Index,
    },

    FutureYieldNotice {
        id: NoticeId,
        parent: C::Hash,
        next_cash_yield: C::Rate,
        next_cash_yield_start_at: C::Timestamp,
        next_cash_yield_index: C::Index,
    },

    SetSupplyCapNotice {
        id: NoticeId,
        parent: C::Hash,
        asset: C::Asset,
        amount: C::Amount,
    },

    ChangeAuthorityNotice {
        id: NoticeId,
        parent: C::Hash,
        new_authorities: Vec<C::PublicKey>,
    },
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Serialize, Deserialize, RuntimeDebug)]
pub enum NoticeStatus<C: Chain> {
    Missing,
    Pending {
        signers: crate::ValidatorSet,
        signatures: Signatures,
        notice: Notice<C>,
    },
    Done,
}

impl<C: Chain> Notice<C> {
    pub fn id(&self) -> NoticeId {
        match self {
            Notice::ExtractionNotice { id, .. } => *id,
            Notice::CashExtractionNotice { id, .. } => *id,
            Notice::FutureYieldNotice { id, .. } => *id,
            Notice::SetSupplyCapNotice { id, .. } => *id,
            Notice::ChangeAuthorityNotice { id, .. } => *id,
        }
    }
}

pub fn encode_ethereum_notice(notice: Notice<Ethereum>) -> Message {
    // XXX
    let chain_ident: Vec<u8> = "ETH".into(); // XX make const?
    let encode_addr =
        |raw: [u8; 20]| -> Vec<u8> { ethabi::encode(&[ethabi::Token::FixedBytes(raw.into())]) };
    // XXX JF: these seem to be relying too much on knowing the struct ab initio, I think we can clean this up a lot
    // XXX most of these eth abi calls are either doing Nothing or padding some bytes, maybe we don't need to use it
    // XXX do this better, maybe figure out how to use dyn Into<ethereum_types::U256> ?
    let encode_int32 = |raw: u32| -> Vec<u8> { ethabi::encode(&[ethabi::Token::Int(raw.into())]) };
    let encode_int128 =
        |raw: u128| -> Vec<u8> { ethabi::encode(&[ethabi::Token::Int(raw.into())]) };

    let encode_public =
        |raw: [u8; 32]| -> Vec<u8> { ethabi::encode(&[ethabi::Token::FixedBytes(raw.into())]) };

    match notice {
        Notice::ExtractionNotice {
            id,
            parent,
            asset,
            account,
            amount,
        } => {
            let era_id: Vec<u8> = encode_int32(id.0);
            let era_index: Vec<u8> = encode_int32(id.1);
            let asset_encoded = encode_addr(asset);
            let amount_encoded = encode_int128(amount); // XXX cast more safely XXX JF: already converted I think
            let account_encoded = encode_addr(account);

            [
                chain_ident,
                era_id,
                era_index,
                parent.into(),
                asset_encoded,
                amount_encoded,
                account_encoded,
            ]
            .concat()
        }
        Notice::CashExtractionNotice {
            id,
            parent,
            account,
            amount,
            cash_yield_index,
        } => {
            let amount_encoded = encode_int128(amount); // XXX cast more safely XXX JF: already converted I think
            [
                chain_ident,
                encode_int32(id.0),
                encode_int32(id.1),
                parent.into(),
                encode_addr(account),
                amount_encoded,
                encode_int128(cash_yield_index),
            ]
            .concat()
        }

        Notice::FutureYieldNotice {
            id,
            parent,
            next_cash_yield,          //: Rate,
            next_cash_yield_start_at, //: Timestamp,
            next_cash_yield_index,    //: Index,
        } => [
            chain_ident,
            encode_int32(id.0),
            encode_int32(id.1),
            parent.into(),
            encode_int128(next_cash_yield),
            encode_int128(next_cash_yield_start_at),
            encode_int128(next_cash_yield_index),
        ]
        .concat(),

        Notice::SetSupplyCapNotice {
            id,     //: NoticeId,
            parent, //: C::Hash,
            asset,  //: C::Asset,
            amount, //: Amount,
        } => {
            let amount_encoded = encode_int128(amount); // XXX cast more safely XXX JF: already converted I think

            [
                chain_ident,
                encode_int32(id.0),
                encode_int32(id.1),
                parent.into(),
                encode_addr(asset),
                amount_encoded,
            ]
            .concat()
        }

        Notice::ChangeAuthorityNotice {
            id,              //: NoticeId,
            parent,          //: C::Hash,
            new_authorities, //: Vec<C::PublicKey>
        } => {
            let authorities_encoded: Vec<Vec<u8>> = new_authorities
                .iter()
                .map(|x| encode_public(x.clone()))
                .collect();
            [
                chain_ident,
                encode_int32(id.0),
                encode_int32(id.1),
                parent.into(),
                authorities_encoded.concat(),
            ]
            .concat()
        }
    }
}

// XXX whats this?
// pub fn to_eth_payload(notice: Notice) -> NoticePayload {
//     let message = encode_ethereum_notice(notice);
//     // TODO: do signer by chain
//     let signer = "0x6a72a2f14577D9Cd0167801EFDd54a07B40d2b61"
//         .as_bytes()
//         .to_vec();
//     NoticePayload {
//         // id: move id,
//         sig: sign(&message),
//         msg: message.to_vec(), // perhaps hex::encode(message)
//         signer: AccountIdent {
//             chain: ChainIdent::Eth,
//             account: signer,
//         },
//     }
// }

/// Helper function to quickly run keccak in the Ethereum-style
fn keccak(input: Vec<u8>) -> [u8; 32] {
    let mut output = [0u8; 32];
    let mut hasher = tiny_keccak::Keccak::v256();
    hasher.update(&input[..]);
    hasher.finalize(&mut output);
    output
}

// TODO: match by chain for signing algorithm or implement as trait
fn sign(message: &Message) -> Signature {
    // TODO: get this from somewhere else
    let not_so_secret: [u8; 32] =
        hex_literal::hex!["50f05592dc31bfc65a77c4cc80f2764ba8f9a7cce29c94a51fe2d70cb5599374"];
    let private_key = secp256k1::SecretKey::parse(&not_so_secret).unwrap();

    let msg = secp256k1::Message::parse(&keccak(message.clone()));
    let x = secp256k1::sign(&msg, &private_key);

    let mut r: [u8; 65] = [0; 65];
    r[0..64].copy_from_slice(&x.0.serialize()[..]);
    r[64] = x.1.serialize();
    sp_core::ecdsa::Signature::from_raw(r).encode()
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_encodes_extraction_notice() {
        let notice = notices::Notice::ExtractionNotice {
            id: (80, 0), // XXX need to keep state of current gen/within gen for each, also parent
            parent: [3u8; 32],
            asset: [2u8; 20],
            amount: 50,
            account: [1u8; 20],
        };

        let expected = [
            69, 84, 72, // chainType::ETH
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 80, // eraID
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, // eraIndex
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, // parent
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, // asset
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 50, // amount
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, // account
        ];

        let encoded = notices::encode_ethereum_notice(notice);
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encodes_cash_extraction_notice() {
        let notice = notices::Notice::CashExtractionNotice {
            id: (80, 0), // XXX need to keep state of current gen/within gen for each, also parent
            parent: [3u8; 32],
            account: [1u8; 20],
            amount: 55,
            cash_yield_index: 75u128,
        };

        let expected = [
            69, 84, 72, // chainType::ETH
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 80, // eraID
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, // eraIndex
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, // parent
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, // account
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 55, // amount
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 75, // cash yield index
        ];

        let encoded = notices::encode_ethereum_notice(notice);
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encodes_future_yield_notice() {
        let notice = notices::Notice::FutureYieldNotice {
            id: (80, 0), // XXX need to keep state of current gen/within gen for each, also parent
            parent: [5u8; 32],
            next_cash_yield: 700u128,
            next_cash_yield_start_at: 200u128,
            next_cash_yield_index: 400u128,
        };

        let expected = [
            69, 84, 72, // chainType::ETH
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 80, // eraID
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, // eraIndex
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            5, 5, 5, // parent
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 2, 188, // next cash yield
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 200, // next cash yield start at
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 1, 144, // next cash yield index
        ];

        let encoded = notices::encode_ethereum_notice(notice);
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encodes_set_supply_cap_notice() {
        let notice = notices::Notice::SetSupplyCapNotice {
            id: (80, 0), // XXX need to keep state of current gen/within gen for each, also parent
            parent: [3u8; 32],
            asset: [70u8; 20],
            amount: 60,
        };

        let expected = [
            69, 84, 72, // chainType::ETH
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 80, // eraID
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, // eraIndex
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, // parent
            70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // asset
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 60, // amount
        ];

        let encoded = notices::encode_ethereum_notice(notice);
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encodes_new_authorities_notice() {
        let notice = notices::Notice::ChangeAuthorityNotice {
            id: (80, 0), // XXX need to keep state of current gen/within gen for each, also parent
            parent: [3u8; 32],
            new_authorities: vec![[6u8; 32], [7u8; 32], [8u8; 32]],
        };

        let expected = [
            69, 84, 72, // chainType::ETH
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 80, // eraID
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, // eraIndex
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, // parent
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            6, 6, 6, // first authority
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, // second
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, // third
        ];

        let encoded = notices::encode_ethereum_notice(notice);
        assert_eq!(encoded, expected);
    }
}