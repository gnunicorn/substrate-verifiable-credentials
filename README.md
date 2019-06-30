# Substrate Verifiable Credentials

A minimal Substrate runtime for verifiable credentials' issuance and verification.

The scenario is of a classroom or workshop where the teachers can issue credentials to students, volunteers and other teacher for attending, assisting with and teaching at the workshop/class respectively.

The inital set of credential issuers are set in the GenesisConfig.

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
  "CredentialType": {
    "_enum": ["Attended", "Conducted", "Volunteered"]
  }
}
```
