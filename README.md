# Substrate Verifiable Credentials

A minimal Substrate runtime for verifiable credentials' issuance and verification.

The inital set of credential issuers are set in the GenesisConfig.

Credentials are issued to holders for subjects. The runtime allows creation of subjects and the identity creating a subject becomes the `issuer` for that `subject`. Credentials can also be revoked by the issuers who issued them.

## Build

Build the WebAssembly binary:

```bash
./scripts/build.sh
```

Build all native code:

```bash
cargo build
```

## Run

You can start a development chain with:

```bash
cargo run -- --dev
```

## Custom Types for UI

```json
{
  "Credential": {
    "subject": "u32",
    "when": "Moment",
    "by": "AccountId"
  }
}
```
