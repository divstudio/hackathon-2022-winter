extern crate alloc;

use crate::types::{asset_value_encode, fp_encode, Asset, AssetValue};
use alloc::{vec::Vec};
use frame_support::{traits::tokens::ExistenceRequirement};
use manta_accounting::transfer;
use manta_pay::{
    config::{self},
    manta_accounting::transfer::{
        canonical::TransferShape,
        receiver::{ReceiverLedger},
        sender::{SenderLedger},
        InvalidSinkAccount, InvalidSourceAccount, SinkPostingKey,
        SourcePostingKey, TransferLedger, TransferLedgerSuperPostingKey, TransferPostingKeyRef,
    },
    manta_parameters::{self, Get as _},
    manta_util::codec::Decode as _,
};
use manta_primitives::{
    assets::{FungibleLedger as _, IsFungible},
};
use crate::{Config, pallet::*};
use crate::common::{PreprocessedEvent, Wrap, WrapPair};
use crate::errors::FungibleLedgerError;
use crate::ledger::ProxyLedger;

pub use crate::types::{Checkpoint, RawCheckpoint, PullResponse};

/// Fungible Token Ledger
pub struct FTTransferLedger<T> where T: Config {
    pub ledger: ProxyLedger<T>
}

impl<T> SenderLedger<config::Parameters> for FTTransferLedger<T>
    where
        T: Config,
{
    type SuperPostingKey = (Wrap<()>, ());
    type ValidUtxoAccumulatorOutput = Wrap<config::UtxoAccumulatorOutput>;
    type ValidNullifier = Wrap<config::Nullifier>;

    fn is_unspent(&self, nullifier: config::Nullifier) -> Option<Self::ValidNullifier> {
        self.ledger.is_unspent(nullifier)
    }

    #[inline]
    fn has_matching_utxo_accumulator_output(
        &self,
        output: config::UtxoAccumulatorOutput,
    ) -> Option<Self::ValidUtxoAccumulatorOutput> {
        self.ledger.has_matching_utxo_accumulator_output(output)
    }

    #[inline]
    fn spend_all<I>(&mut self, super_key: &Self::SuperPostingKey, iter: I)
        where
            I: IntoIterator<Item = (Self::ValidUtxoAccumulatorOutput, Self::ValidNullifier)>,
    {
        self.ledger.spend_all(super_key, iter)
    }
}

impl<T> ReceiverLedger<config::Parameters> for FTTransferLedger<T>
    where
        T: Config,
{
    type SuperPostingKey = (Wrap<()>, ());
    type ValidUtxo = Wrap<config::Utxo>;

    #[inline]
    fn is_not_registered(&self, utxo: config::Utxo) -> Option<Self::ValidUtxo> {
        self.ledger.is_not_registered(utxo)
    }

    #[inline]
    fn register_all<I>(&mut self, super_key: &Self::SuperPostingKey, iter: I)
        where
            I: IntoIterator<Item=(Self::ValidUtxo, config::Note)>,
    {
        self.ledger.register_all(super_key, iter)
    }
}

impl<T> TransferLedger<config::Config> for FTTransferLedger<T>
    where
        T: Config,
{
    type SuperPostingKey = ();
    type AccountId = T::AccountId;
    type Event = PreprocessedEvent<T>;
    type UpdateError = FungibleLedgerError;
    type ValidSourceAccount = WrapPair<Self::AccountId, AssetValue>;
    type ValidSinkAccount = WrapPair<Self::AccountId, AssetValue>;
    type ValidProof = Wrap<()>;

    #[inline]
    fn check_source_accounts<I>(
        &self,
        asset_id: &config::AssetId,
        sources: I,
    ) -> Result<Vec<Self::ValidSourceAccount>, InvalidSourceAccount<config::Config, Self::AccountId>>
        where
            I: Iterator<Item = (Self::AccountId, config::AssetValue)>,
    {
        let metadata = Pallet::<T>::get_metadata(*asset_id).expect("Metadata get failed");
        let id = metadata.get_fungible_id().expect("Asset Id get failed");
        sources
            .map(move |(account_id, withdraw)| {
                FungibleLedger::<T>::can_withdraw(
                    id.clone(),
                    &account_id,
                    &withdraw,
                    ExistenceRequirement::KeepAlive,
                )
                    .map(|_| WrapPair(account_id.clone(), withdraw))
                    .map_err(|_| InvalidSourceAccount {
                        account_id,
                        asset_id: *asset_id,
                        withdraw,
                    })
            })
            .collect()
    }

    #[inline]
    fn check_sink_accounts<I>(
        &self,
        asset_id: &config::AssetId,
        sinks: I,
    ) -> Result<Vec<Self::ValidSinkAccount>, InvalidSinkAccount<config::Config, Self::AccountId>>
        where
            I: Iterator<Item = (Self::AccountId, config::AssetValue)>,
    {
        let metadata = Pallet::<T>::get_metadata(*asset_id).expect("Metadata get failed");
        // NOTE: Existence of accounts is type-checked so we don't need to do anything here, just
        // pass the data forward.
        let id = metadata.get_fungible_id().expect("Asset Id get failed");
        sinks
            .map(move |(account_id, deposit)| {
                FungibleLedger::<T>::can_deposit(
                    id.clone(),
                    &account_id,
                    deposit,
                    false,
                )
                    .map(|_| WrapPair(account_id.clone(), deposit))
                    .map_err(|_| InvalidSinkAccount {
                        account_id,
                        asset_id: *asset_id,
                        deposit,
                    })
            })
            .collect()
    }

    #[inline]
    fn is_valid(
        &self,
        posting_key: TransferPostingKeyRef<config::Config, Self>,
    ) -> Option<(Self::ValidProof, Self::Event)> {
        let (mut verifying_context, event) =
            match TransferShape::from_posting_key_ref(&posting_key)? {
                TransferShape::ToPrivate => (
                    manta_parameters::pay::testnet::verifying::ToPrivate::get()
                        .expect("Checksum did not match."),
                    PreprocessedEvent::<T>::ToPrivate {
                        asset: Asset::new(
                            fp_encode(posting_key.asset_id.or(None)?).ok()?,
                            asset_value_encode(posting_key.sources[0].1),
                        ),
                        source: posting_key.sources[0].0.clone(),
                    },
                ),
                TransferShape::PrivateTransfer => (
                    manta_parameters::pay::testnet::verifying::PrivateTransfer::get()
                        .expect("Checksum did not match."),
                    PreprocessedEvent::<T>::PrivateTransfer,
                ),
                TransferShape::ToPublic => (
                    manta_parameters::pay::testnet::verifying::ToPublic::get()
                        .expect("Checksum did not match."),
                    PreprocessedEvent::<T>::ToPublic {
                        asset: Asset::new(
                            fp_encode(posting_key.asset_id.or(None)?).ok()?,
                            asset_value_encode(posting_key.sinks[0].1),
                        ),
                        sink: posting_key.sinks[0].0.clone(),
                    },
                ),
            };
        posting_key
            .has_valid_proof(
                &config::VerifyingContext::decode(&mut verifying_context)
                    .expect("Unable to decode the verifying context."),
            )
            .ok()?
            .then_some((Wrap(()), event))
    }

    #[inline]
    fn update_public_balances(
        &mut self,
        _asset_type: transfer::AssetType,
        super_key: &TransferLedgerSuperPostingKey<config::Config, Self>,
        asset_id: config::AssetId,
        sources: Vec<SourcePostingKey<config::Config, Self>>,
        sinks: Vec<SinkPostingKey<config::Config, Self>>,
        proof: Self::ValidProof,
    ) -> Result<(), Self::UpdateError> {
        let _ = (proof, super_key);
        let metadata = Pallet::<T>::get_metadata(asset_id)?;
        let id = metadata
            .get_fungible_id()
            .ok_or(FungibleLedgerError::UnknownAsset)?;
        for WrapPair(account_id, withdraw) in sources {
            FungibleLedger::<T>::transfer(
                id.clone(),
                &account_id,
                &Pallet::<T>::account_id(),
                withdraw,
                ExistenceRequirement::KeepAlive,
            )?;
        }
        for WrapPair(account_id, deposit) in sinks {
            FungibleLedger::<T>::transfer(
                id.clone(),
                &Pallet::<T>::account_id(),
                &account_id,
                deposit,
                ExistenceRequirement::KeepAlive,
            )?;
        }
        Ok(())
    }
}