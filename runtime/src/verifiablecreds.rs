use parity_codec::{Decode, Encode};
use runtime_primitives::traits::Hash;
use support::{decl_event, decl_module, decl_storage, StorageMap, StorageValue};
use system::ensure_signed;

pub trait Trait: system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as VerifiableCreds {
        // Issuers.
        Issuers get(issuers) config(): map T::AccountId => bool;
        // Credentials store.
        // Mapping (holder, subject) to (issuer, timestamp).
        Credentials get(credentials): map (T::AccountId, T::Hash) => (T::AccountId, T::Moment);
    }
    extra_genesis_skip_phantom_data_field;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Hash = <T as system::Trait>::Hash,
    {
        // A credential is issued - holder, cred, issuer
        CredentialIssued(AccountId, Hash, AccountId),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        pub fn issue_credential(origin, to: T::AccountId, credential: T::Hash) {
            // Check if origin is an issuer.
            // Issue the credential - add to storage.
        }

        pub fn verify_credential(origin, credential: T::Hash) {
            // Query storage for origin and credential.
        }
    }
}
