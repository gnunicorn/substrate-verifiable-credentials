use support::{decl_event, decl_module, decl_storage, StorageMap, StorageValue, ensure};
use system::ensure_signed;
use parity_codec::{Decode, Encode};

pub trait Trait: system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Default, PartialEq)]
pub struct Credential<Timestamp, AccountId> {
   subject: u32,
   when: Timestamp,
   by: AccountId
}

decl_storage! {
    trait Store for Module<T: Trait> as VerifiableCreds {
        // global nonce for subject count
        SubjectNonce get(subject_nonce) config(): u32;
        // Issuers can issue credentials to others.
        // Issuer to Subject mapping.
        Subjects get(subjects) config(): map u32 => T::AccountId;
        // Credentials store.
        // Mapping (holder, subject) to Credential.
        Credentials get(credentials): map (T::AccountId, u32) => Credential<T::Moment, T::AccountId>;
    }
    extra_genesis_skip_phantom_data_field;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        // A credential is issued - holder, subj, issuer
        CredentialIssued(AccountId, u32, AccountId),
        // A credential is revoked - holder, subj, issuer
        CredentialRevoked(AccountId, u32, AccountId),
        // A new subject is created.
        SubjectCreated(AccountId, u32),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        /// Issue a credential to an identity.
        /// Only an issuer can call this function.
        pub fn issue_credential(origin, to: T::AccountId, subject: u32) {
            // Check if origin is an issuer.
            // Issue the credential - add to storage.

            let sender = ensure_signed(origin)?;
            let subject_issuer = Self::subjects(subject);
            ensure!(subject_issuer == sender, "Unauthorized.");

            let now = <timestamp::Module<T>>::get();
            let cred = Credential {
              subject,
              when: now,
              by: sender.clone()
            };

            <Credentials<T>>::insert((to.clone(), subject), cred);

            Self::deposit_event(RawEvent::CredentialIssued(to, subject, sender));
        }

        /// Revoke a credential.
        /// Only an issuer can call this function. 
        pub fn revoke_credential(origin, to: T::AccountId, subject: u32) {
            // Check if origin is an issuer.
            // Check if credential is issued.
            // Change the bool flag of the stored credential tuple to false.

            let sender = ensure_signed(origin)?;
            let subject_issuer = Self::subjects(subject);
            ensure!(subject_issuer == sender, "Unauthorized.");
            ensure!(<Credentials<T>>::exists((to.clone(), subject)), "Credential not issued yet.");

            <Credentials<T>>::remove((to.clone(), subject));
            Self::deposit_event(RawEvent::CredentialRevoked(to, subject, sender));
        }

        /// Verify a credential.
        pub fn verify_credential(origin, holder: T::AccountId, subject: u32) {
            let _sender = ensure_signed(origin)?;

            // Ensure credential is issued and allowed to be verified.
            ensure!(<Credentials<T>>::exists((holder.clone(), subject)), "Credential not issued yet.");
        }

        /// Create a new subject.
        pub fn create_subject(origin) {
            let sender = ensure_signed(origin)?;
            let subject_nonce = <SubjectNonce<T>>::get();

            <Subjects<T>>::insert(subject_nonce, sender.clone());

            // Update the subject nonce.
            <SubjectNonce<T>>::put(subject_nonce + 1);

            // Deposit the event.
            Self::deposit_event(RawEvent::SubjectCreated(sender, subject_nonce));
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
        issuers: vec![(1, 1), (2, 2)],
        subject_nonce: 3,
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
            VerifiableCreds::issue_credential(Origin::signed(1), 3, 2),
            "Unauthorized.");
    });
  }

  #[test]
  fn should_issue() {
    with_externalities(&mut new_test_ext(), || {
        assert_ok!(
            VerifiableCreds::issue_credential(Origin::signed(1), 3, 1));
    });
  }

  #[test]
  fn should_revoke() {
    with_externalities(&mut new_test_ext(), || {
        assert_ok!(
            VerifiableCreds::issue_credential(Origin::signed(1), 3, 1));
        assert_ok!(
            VerifiableCreds::revoke_credential(Origin::signed(1), 3, 1));
    });
  }

  #[test]
  fn should_add_subject() {
    with_externalities(&mut new_test_ext(), || {
        assert_ok!(
            VerifiableCreds::create_subject(Origin::signed(3)));
        assert_eq!(
            VerifiableCreds::issuers(3), 3);
    });
  }

  #[test]
  fn should_issue_new_subject() {
    with_externalities(&mut new_test_ext(), || {
        assert_ok!(
            VerifiableCreds::create_subject(Origin::signed(3)));
        assert_eq!(
            VerifiableCreds::issuers(3), 3);
        assert_ok!(
            VerifiableCreds::issue_credential(Origin::signed(3), 4, 3));
    });
  }
}