use support::{decl_event, decl_module, decl_storage, StorageMap, ensure};
use system::ensure_signed;

pub trait Trait: system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as VerifiableCreds {
        // Issuers.
        Issuers get(issuers) config(): map T::AccountId => bool;
        // Credentials store.
        // Mapping (holder, subject) to (issuer, timestamp, is_valid).
        Credentials get(credentials): map (T::AccountId, T::Hash) => (T::AccountId, T::Moment, bool);
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

        /// Issue a credential to an identity.
        /// Only an issuer can call this function.
        pub fn issue_credential(origin, to: T::AccountId, credential: T::Hash) {
            // Check if origin is an issuer.
            // Issue the credential - add to storage.

            let sender = ensure_signed(origin)?;
            ensure!(Self::issuers(sender.clone()), "Unauthorized.");

            let now = <timestamp::Module<T>>::get();
            <Credentials<T>>::insert((to.clone(), credential.clone()), (sender.clone(), now, true));

            Self::deposit_event(RawEvent::CredentialIssued(to, credential, sender));
        }

        /// Revoke a credential.
        /// Only an issuer can call this function. 
        pub fn revoke_credential(origin, to: T::AccountId, credential: T::Hash) {
            // Check if origin is an issuer.
            // Check if credential is issued.
            // Change the bool flag of the stored credential tuple to false.

            let sender = ensure_signed(origin)?;
            ensure!(<Issuers<T>>::exists(sender.clone()), "Unauthorized.");
            ensure!(<Credentials<T>>::exists((to.clone(), credential.clone())), "Credential not issued yet.");

            <Credentials<T>>::mutate((to, credential), |v| { v.2 = false } );
        }

        /// Verify a credential.
        /// Only an allowed verifier can verify.
        pub fn verify_credential(origin, holder: T::AccountId, credential: T::Hash) {
            let sender = ensure_signed(origin)?;

            // Ensure credential is issued and allowed to be verified.
            ensure!(<Credentials<T>>::exists((holder.clone(), credential.clone())), "Credential not issued yet.");

            let cred = <Credentials<T>>::get((sender.clone(), credential.clone()));
            ensure!(cred.2 == true, "Credential not valid.");
        }
    }
}
