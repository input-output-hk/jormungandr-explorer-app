mod utils;

use chain::{account, certificate, fee, key, transaction as tx, txbuilder, value};
use chain_crypto as crypto;
use chain_impl_mockchain as chain;
use crypto::bech32::Bech32;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct PrivateKey(key::EitherEd25519SecretKey);

impl From<key::EitherEd25519SecretKey> for PrivateKey {
    fn from(secret_key: key::EitherEd25519SecretKey) -> PrivateKey {
        PrivateKey(secret_key)
    }
}

#[wasm_bindgen]
impl PrivateKey {
    pub fn from_bech32(bech32_str: &str) -> Result<PrivateKey, JsValue> {
        crypto::SecretKey::try_from_bech32_str(&bech32_str)
            .map(|key| key::EitherEd25519SecretKey::Extended(key))
            .or(crypto::SecretKey::try_from_bech32_str(&bech32_str)
                .map(|key| key::EitherEd25519SecretKey::Normal(key)))
            .map(|secret_key| PrivateKey(secret_key))
            .map_err(|_| JsValue::from_str("Invalid secret key"))
    }
}

#[wasm_bindgen]
pub struct PublicKey(crypto::Ed25519);

//-----------------------------//
//----------Address------------//
//-----------------------------//

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Address(chain_addr::Address);

#[wasm_bindgen]
impl Address {
    //XXX: Maybe this should be from_bech32?
    pub fn from_string(s: &str) -> Result<Address, JsValue> {
        chain_addr::AddressReadable::from_string(s)
            .map(|address_readable| Address(address_readable.to_address()))
            .map_err(|e| JsValue::from_str(&format! {"{:?}", e}))
    }

    pub fn to_string(&self) -> String {
        format!("{}", chain_addr::AddressReadable::from_address(&self.0))
    }
}

impl From<chain_addr::Address> for Address {
    fn from(address: chain_addr::Address) -> Address {
        Address(address)
    }
}

//-----------------------------------//
//-------- Transaction --------------//
//-----------------------------------//

// the wrapper is because non C-enums cannot be directly exposed
// and the enum is because parametrized types cannot be exposed either
#[wasm_bindgen]
pub struct Transaction(EitherTransaction);

enum EitherTransaction {
    TransactionWithoutCertificate(tx::Transaction<chain_addr::Address, tx::NoExtra>),
    TransactionWithCertificate(tx::Transaction<chain_addr::Address, certificate::Certificate>),
}

impl From<tx::Transaction<chain_addr::Address, tx::NoExtra>> for Transaction {
    fn from(tx: tx::Transaction<chain_addr::Address, tx::NoExtra>) -> Self {
        Transaction(EitherTransaction::TransactionWithoutCertificate(tx))
    }
}

impl From<tx::Transaction<chain_addr::Address, certificate::Certificate>> for Transaction {
    fn from(tx: tx::Transaction<chain_addr::Address, certificate::Certificate>) -> Self {
        Transaction(EitherTransaction::TransactionWithCertificate(tx))
    }
}

#[wasm_bindgen]
impl Transaction {
    pub fn id(&self) -> TransactionId {
        match &self.0 {
            EitherTransaction::TransactionWithoutCertificate(tx) => tx.hash(),
            EitherTransaction::TransactionWithCertificate(tx) => tx.hash(),
        }
        .into()
    }
}

//-----------------------------------//
//--------TransactionBuilder---------//
//-----------------------------------//

#[wasm_bindgen]
pub struct TransactionBuilder(EitherTransactionBuilder);

enum EitherTransactionBuilder {
    TransactionBuilderNoExtra(txbuilder::TransactionBuilder<chain_addr::Address, tx::NoExtra>),
    TransactionBuilderCertificate(
        txbuilder::TransactionBuilder<chain_addr::Address, certificate::Certificate>,
    ),
}

impl From<txbuilder::TransactionBuilder<chain_addr::Address, tx::NoExtra>> for TransactionBuilder {
    fn from(builder: txbuilder::TransactionBuilder<chain_addr::Address, tx::NoExtra>) -> Self {
        TransactionBuilder(EitherTransactionBuilder::TransactionBuilderNoExtra(builder))
    }
}

impl From<txbuilder::TransactionBuilder<chain_addr::Address, certificate::Certificate>>
    for TransactionBuilder
{
    fn from(
        builder: txbuilder::TransactionBuilder<chain_addr::Address, certificate::Certificate>,
    ) -> Self {
        TransactionBuilder(EitherTransactionBuilder::TransactionBuilderCertificate(
            builder,
        ))
    }
}

#[wasm_bindgen]
impl TransactionBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        txbuilder::TransactionBuilder::new().into()
    }

    #[wasm_bindgen]
    pub fn set_sertificate(&mut self, certificate: Certificate) -> Result<(), JsValue> {
        let builder = match &self.0 {
            EitherTransactionBuilder::TransactionBuilderNoExtra(ref builder) => {
                builder.clone().set_certificate(certificate.0)
            }
            EitherTransactionBuilder::TransactionBuilderCertificate(_) =>
            //Is either this or replacing the extra
            {
                return Err(JsValue::from_str("There is already one certificate"))
            }
        };
        self.0 = EitherTransactionBuilder::TransactionBuilderCertificate(builder);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn add_input(&mut self, input: Input) {
        match &mut self.0 {
            EitherTransactionBuilder::TransactionBuilderNoExtra(ref mut builder) => {
                builder.add_input(&input.0)
            }
            EitherTransactionBuilder::TransactionBuilderCertificate(ref mut builder) => {
                builder.add_input(&input.0)
            }
        }
    }

    #[wasm_bindgen]
    pub fn add_output(&mut self, address: Address, value: Value) {
        match &mut self.0 {
            EitherTransactionBuilder::TransactionBuilderNoExtra(ref mut builder) => {
                builder.add_output(address.0, value.0)
            }
            EitherTransactionBuilder::TransactionBuilderCertificate(ref mut builder) => {
                builder.add_output(address.0, value.0)
            }
        }
    }

    #[wasm_bindgen]
    pub fn estimate_fee(&self, fee: &Fee) -> Result<Value, JsValue> {
        let fee_algorithm = match fee.0 {
            FeeVariant::Linear(fee_algorithm) => fee_algorithm,
        };
        match &self.0 {
            EitherTransactionBuilder::TransactionBuilderNoExtra(ref builder) => {
                builder.estimate_fee(fee_algorithm)
            }
            EitherTransactionBuilder::TransactionBuilderCertificate(ref builder) => {
                builder.estimate_fee(fee_algorithm)
            }
        }
        .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
        .map(|value| value.into())
    }

    #[wasm_bindgen]
    pub fn get_balance(&self, fee: &Fee) -> Result<Balance, JsValue> {
        let fee_algorithm = match fee.0 {
            FeeVariant::Linear(fee_algorithm) => fee_algorithm,
        };
        match &self.0 {
            EitherTransactionBuilder::TransactionBuilderNoExtra(ref builder) => {
                builder.get_balance(fee_algorithm)
            }
            EitherTransactionBuilder::TransactionBuilderCertificate(ref builder) => {
                builder.get_balance(fee_algorithm)
            }
        }
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
        .map(|balance| balance.into())
    }

    #[wasm_bindgen]
    pub fn get_balance_without_fee(&self) -> Result<Balance, JsValue> {
        match &self.0 {
            EitherTransactionBuilder::TransactionBuilderNoExtra(ref builder) => {
                builder.get_balance_without_fee()
            }
            EitherTransactionBuilder::TransactionBuilderCertificate(ref builder) => {
                builder.get_balance_without_fee()
            }
        }
        .map(|balance| balance.into())
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    #[wasm_bindgen]
    pub fn unchecked_finalize(self) -> Transaction {
        match self.0 {
            EitherTransactionBuilder::TransactionBuilderNoExtra(builder) => builder.tx.into(),
            EitherTransactionBuilder::TransactionBuilderCertificate(builder) => builder.tx.into(),
        }
    }

    #[wasm_bindgen]
    pub fn finalize(self, fee: &Fee, output_policy: OutputPolicy) -> Result<Transaction, JsValue> {
        let fee_algorithm = match fee.0 {
            FeeVariant::Linear(fee_algorithm) => fee_algorithm,
        };

        match self.0 {
            EitherTransactionBuilder::TransactionBuilderNoExtra(builder) => builder
                .finalize(fee_algorithm, output_policy.0)
                .map(|(_, tx)| tx.into()),
            EitherTransactionBuilder::TransactionBuilderCertificate(builder) => builder
                .finalize(fee_algorithm, output_policy.0)
                .map(|(_, tx)| tx.into()),
        }
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
}

#[wasm_bindgen]
pub struct OutputPolicy(txbuilder::OutputPolicy);

impl From<txbuilder::OutputPolicy> for OutputPolicy {
    fn from(output_policy: txbuilder::OutputPolicy) -> OutputPolicy {
        OutputPolicy(output_policy)
    }
}

#[wasm_bindgen]
impl OutputPolicy {
    pub fn forget() -> OutputPolicy {
        txbuilder::OutputPolicy::Forget.into()
    }

    pub fn one(address: Address) -> OutputPolicy {
        txbuilder::OutputPolicy::One(address.0).into()
    }
}

#[wasm_bindgen]
pub struct TransactionFinalizer(txbuilder::TransactionFinalizer);

impl From<txbuilder::TransactionFinalizer> for TransactionFinalizer {
    fn from(finalizer: txbuilder::TransactionFinalizer) -> TransactionFinalizer {
        TransactionFinalizer(finalizer)
    }
}

impl TransactionFinalizer {
    pub fn new(transaction: Transaction) -> Self {
        TransactionFinalizer(match transaction.0 {
            EitherTransaction::TransactionWithCertificate(tx) => {
                txbuilder::TransactionFinalizer::new_cert(tx)
            }
            EitherTransaction::TransactionWithoutCertificate(tx) => {
                txbuilder::TransactionFinalizer::new_trans(tx)
            }
        })
    }

    pub fn set_witness(&mut self, index: usize, witness: Witness) -> Result<(), JsValue> {
        self.0
            .set_witness(index, witness.0)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    pub fn get_txid(&self) -> TransactionId {
        self.0.get_txid().into()
    }

    pub fn build(self) -> Result<txbuilder::GeneratedTransaction, JsValue> {
        self.0
            .build()
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
}

#[wasm_bindgen]
pub struct GeneratedTransaction(txbuilder::GeneratedTransaction);

impl From<txbuilder::GeneratedTransaction> for GeneratedTransaction {
    fn from(generated_transaction: txbuilder::GeneratedTransaction) -> GeneratedTransaction {
        GeneratedTransaction(generated_transaction)
    }
}

#[wasm_bindgen]
pub struct TransactionId(tx::TransactionId);

#[wasm_bindgen]
impl TransactionId {
    pub fn from_bytes(bytes: &[u8]) -> TransactionId {
        tx::TransactionId::hash_bytes(bytes).into()
    }

    pub fn from_hex(input: &str) -> Result<TransactionId, JsValue> {
        tx::TransactionId::from_str(input)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
            .map(TransactionId)
    }
}

impl From<tx::TransactionId> for TransactionId {
    fn from(txid: tx::TransactionId) -> TransactionId {
        TransactionId(txid)
    }
}

#[wasm_bindgen]
pub struct Hash(key::Hash);

impl From<key::Hash> for Hash {
    fn from(hash: key::Hash) -> Hash {
        Hash(hash)
    }
}

#[wasm_bindgen]
impl Hash {
    pub fn from_bytes(bytes: &[u8]) -> Hash {
        key::Hash::hash_bytes(bytes).into()
    }
}

#[wasm_bindgen]
pub struct Input(tx::Input);

impl From<tx::Input> for Input {
    fn from(input: tx::Input) -> Input {
        Input(input)
    }
}

#[wasm_bindgen]
impl Input {
    pub fn from_utxo(utxo_pointer: &UtxoPointer) -> Self {
        Input(tx::Input::from_utxo(utxo_pointer.0.clone()))
    }

    pub fn from_account(account: &Account, v: u64) -> Self {
        Input(tx::Input::from_account(account.0.clone(), value::Value(v)))
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct UtxoPointer(tx::UtxoPointer);

#[wasm_bindgen]
impl UtxoPointer {
    pub fn new(tx_id: TransactionId, output_index: u8, value: u64) -> UtxoPointer {
        UtxoPointer(tx::UtxoPointer {
            transaction_id: tx_id.0,
            output_index,
            value: value::Value(value),
        })
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct Account(tx::AccountIdentifier);

impl From<tx::AccountIdentifier> for Account {
    fn from(account_identifier: tx::AccountIdentifier) -> Account {
        Account(account_identifier)
    }
}

#[wasm_bindgen]
impl Account {
    pub fn from_address(address: &Address) -> Result<Account, JsValue> {
        if let chain_addr::Kind::Account(key) = address.0.kind() {
            Ok(Account(tx::AccountIdentifier::from_single_account(
                key.clone().into(),
            )))
        } else {
            Err(JsValue::from_str("Address is not account"))
        }
    }
}

#[wasm_bindgen]
pub struct Output(tx::Output<chain_addr::Address>);

impl From<tx::Output<chain_addr::Address>> for Output {
    fn from(output: tx::Output<chain_addr::Address>) -> Output {
        Output(output)
    }
}

#[wasm_bindgen]
#[derive(Debug, Eq, PartialEq)]
pub struct Value(value::Value);

#[wasm_bindgen]
impl Value {
    //It probably would be better to derive deserialize?
    pub fn from_u64(number: u64) -> Self {
        Value(value::Value(number))
    }
}

impl From<value::Value> for Value {
    fn from(value: value::Value) -> Value {
        Value(value)
    }
}

#[wasm_bindgen]
pub struct Certificate(certificate::Certificate);

impl From<certificate::Certificate> for Certificate {
    fn from(certificate: certificate::Certificate) -> Certificate {
        Certificate(certificate)
    }
}

#[wasm_bindgen]
pub struct Balance(tx::Balance);

impl From<tx::Balance> for Balance {
    fn from(balance: tx::Balance) -> Balance {
        Balance(balance)
    }
}

#[wasm_bindgen]
impl Balance {
    //Not sure is this is the best way
    pub fn get_sign(&self) -> JsValue {
        JsValue::from_str(match self.0 {
            tx::Balance::Positive(_) => "positive",
            tx::Balance::Negative(_) => "negative",
            tx::Balance::Zero => "zero",
        })
    }

    pub fn get_value(&self) -> Value {
        match self.0 {
            tx::Balance::Positive(v) => Value(v),
            tx::Balance::Negative(v) => Value(v),
            tx::Balance::Zero => Value(value::Value(0)),
        }
    }
}

#[wasm_bindgen]
pub struct Fee(FeeVariant);

#[wasm_bindgen]
impl Fee {
    pub fn linear_fee(constant: u64, coefficient: u64, certificate: u64) -> Fee {
        Fee(FeeVariant::Linear(fee::LinearFee::new(
            constant,
            coefficient,
            certificate,
        )))
    }
}

pub enum FeeVariant {
    Linear(fee::LinearFee),
}

#[wasm_bindgen]
pub struct Witness(tx::Witness);

#[wasm_bindgen]
impl Witness {
    pub fn for_utxo(
        genesis_hash: Hash,
        transaction_id: TransactionId,
        secret_key: PrivateKey,
    ) -> Witness {
        Witness(tx::Witness::new_utxo(
            &genesis_hash.0,
            &transaction_id.0,
            &secret_key.0,
        ))
    }

    pub fn for_account(
        genesis_hash: Hash,
        transaction_id: TransactionId,
        secret_key: PrivateKey,
        account_spending_counter: SpendingCounter,
    ) -> Witness {
        Witness(tx::Witness::new_account(
            &genesis_hash.0,
            &transaction_id.0,
            &account_spending_counter.0,
            &secret_key.0,
        ))
    }
}

#[wasm_bindgen]
pub struct SpendingCounter(account::SpendingCounter);

impl From<account::SpendingCounter> for SpendingCounter {
    fn from(spending_counter: account::SpendingCounter) -> SpendingCounter {
        SpendingCounter(spending_counter)
    }
}

#[wasm_bindgen]
impl SpendingCounter {
    pub fn zero() -> Self {
        account::SpendingCounter::zero().into()
    }

    pub fn from_u32(counter: u32) -> Self {
        account::SpendingCounter::from(counter).into()
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;