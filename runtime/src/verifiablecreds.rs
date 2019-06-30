use support::{decl_event, decl_module, decl_storage, StorageMap, ensure};
use system::ensure_signed;
use parity_codec::{Decode, Encode};

pub trait Trait: system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

/// The CredentialType - In context of workshop attendance
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub enum CredentialType {
    Attended,
    Conducted,
    Volunteered
}

impl Default for CredentialType {
    fn default() -> Self { CredentialType::Attended }
}

decl_storage! {
    trait Store for Module<T: Trait> as VerifiableCreds {
        // Issuers can issue credentials to others.
        Issuers get(issuers) config(): map T::AccountId => bool;
        // Credentials store.
        // Mapping (holder, subject) to (issuer, timestamp, is_valid).
        Credentials get(credentials): map (T::AccountId, CredentialType) => (T::AccountId, T::Moment, bool);
    }
    extra_genesis_skip_phantom_data_field;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        // A credential is issued - holder, cred, issuer
        CredentialIssued(AccountId, CredentialType, AccountId),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        /// Issue a credential to an identity.
        /// Only an issuer can call this function.
        pub fn issue_credential(origin, to: T::AccountId, credential: CredentialType) {
            // Check if origin is an issuer.
            // Issue the credential - add to storage.

            let sender = ensure_signed(origin)?;
            ensure!(Self::issuers(sender.clone()), "Unauthorized.");

            let now = <timestamp::Module<T>>::get();
            <Credentials<T>>::insert((to.clone(), credential.clone()), (sender.clone(), now, true));

            // If credential is of the type `Conducted` the workshop,
            // add the holder as an issuer too.
            if credential == CredentialType::Conducted {
                 <Issuers<T>>::insert(to.clone(), true);
            }

            Self::deposit_event(RawEvent::CredentialIssued(to, credential, sender));
        }

        /// Revoke a credential.
        /// Only an issuer can call this function. 
        pub fn revoke_credential(origin, to: T::AccountId, credential: CredentialType) {
            // Check if origin is an issuer.
            // Check if credential is issued.
            // Change the bool flag of the stored credential tuple to false.

            let sender = ensure_signed(origin)?;
            ensure!(<Issuers<T>>::exists(sender.clone()), "Unauthorized.");
            ensure!(<Credentials<T>>::exists((to.clone(), credential.clone())), "Credential not issued yet.");

            <Credentials<T>>::mutate((to.clone(), credential.clone()), |v| { v.2 = false } );

            // If credential is of the type `Conducted` the workshop,
            // remove the holder as an issuer.
            if credential == CredentialType::Conducted {
                 <Issuers<T>>::remove(to.clone());
            }
        }

        /// Verify a credential.
        /// Only an allowed verifier can verify.
        pub fn verify_credential(origin, holder: T::AccountId, credential: CredentialType) {
            let sender = ensure_signed(origin)?;

            // Ensure credential is issued and allowed to be verified.
            ensure!(<Credentials<T>>::exists((holder.clone(), credential.clone())), "Credential not issued yet.");

            let cred = <Credentials<T>>::get((sender.clone(), credential.clone()));
            ensure!(cred.2 == true, "Credential not valid.");
        }
    }
}

#[cfg(test)]
mod tests {
  use super::*;

  use primitives::{Blake2Hasher, H256};
  use runtime_io::with_externalities;
  use runtime_primitives::{
    testing::{Digest, DigestItem, Header},
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
  };
  use support::{assert_noop, assert_ok, impl_outer_origin};

  impl_outer_origin! {
    pub enum Origin for Test {}
  }

  // For testing the module, we construct a mock runtime. This means
  // first constructing a configuration type (`Test`) which implements each of the
  // configuration traits of modules we use.
  #[derive(Clone, Eq, PartialEq)]
  pub struct Test;
  impl system::Trait for Test {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Digest = Digest;
    type AccountId = u64;
    type Lookup = IdentityLookup<u64>;
    type Header = Header;
    type Event = ();
    type Log = DigestItem;
  }
  impl timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
  }
  impl Trait for Test {
    type Event = ();
  }
  type VerifiableCreds = Module<Test>;

  // builds the genesis config store and sets mock values
  fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
    let mut t = system::GenesisConfig::<Test>::default()
      .build_storage()
      .unwrap()
      .0;
    t.extend(
      GenesisConfig::<Test> {
        issuers: vec![(1, true), (2, true)],
      }
      .build_storage()
      .unwrap()
      .0,
    );
    t.into()
  }

  #[test]
  fn should_fail_issue() {
    with_externalities(&mut new_test_ext(), || {
        assert_noop!(
            VerifiableCreds::issue_credential(Origin::signed(4), 3, CredentialType::Attended),
            "Unauthorized.");
    });
  }

  #[test]
  fn should_issue() {
    with_externalities(&mut new_test_ext(), || {
        assert_ok!(
            VerifiableCreds::issue_credential(Origin::signed(1), 3, CredentialType::Attended));
    });
  }

  #[test]
  fn should_revoke() {
    with_externalities(&mut new_test_ext(), || {
        assert_ok!(
            VerifiableCreds::issue_credential(Origin::signed(1), 3, CredentialType::Attended));
        assert_ok!(
            VerifiableCreds::revoke_credential(Origin::signed(1), 3, CredentialType::Attended));
    });
  }

  #[test]
  fn should_not_add_issuer() {
    with_externalities(&mut new_test_ext(), || {
        assert_ok!(
            VerifiableCreds::issue_credential(Origin::signed(1), 3, CredentialType::Attended));
        assert_noop!(
            VerifiableCreds::issue_credential(Origin::signed(3), 3, CredentialType::Conducted),
            "Unauthorized.");
    });
  }

  #[test]
  fn should_add_issuer() {
    with_externalities(&mut new_test_ext(), || {
        assert_ok!(
            VerifiableCreds::issue_credential(Origin::signed(1), 3, CredentialType::Conducted));
        assert_eq!(
            VerifiableCreds::issuers(3), true);
    });
  }

  #[test]
  fn should_remove_issuer() {
    with_externalities(&mut new_test_ext(), || {
        assert_ok!(
            VerifiableCreds::issue_credential(Origin::signed(1), 3, CredentialType::Conducted));
        assert_ok!(
            VerifiableCreds::revoke_credential(Origin::signed(1), 3, CredentialType::Conducted));
        assert_eq!(
            VerifiableCreds::issuers(3), false);
    });
  }

  #[test]
  fn should_add_issuer_new_issuer() {
    with_externalities(&mut new_test_ext(), || {
        assert_ok!(
            VerifiableCreds::issue_credential(Origin::signed(1), 3, CredentialType::Conducted));
        assert_eq!(
            VerifiableCreds::issuers(3), true);
        assert_ok!(
            VerifiableCreds::issue_credential(Origin::signed(3), 4, CredentialType::Conducted));
        assert_eq!(
            VerifiableCreds::issuers(4), true);
    });
  }
}