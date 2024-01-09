# tonic-derive-error

A macro-based utility that simplifies Tonic error handling.

## Example

```rust
use thiserror::Error;
use tonic::Code;
use tonic_derive_error::GrpcError;

#[derive(Error, GrpcError, Debug)]
pub enum CustomError {
    #[error("something went wrong")]
    #[grpc_error(status = "Code::FailedPrecondition")]
    ErrorVariant
}
```

`GrpcError` derive automatically generates the `From<CustomError> for tonic::Status` trait implementation.

When debug assertions are disabled (e.g. in release builds) errors with the `Code::Internal` status code
are not displayed in a response and instead are logged using the `tracing` crate.

## License

Licensed under either of Apache License 2.0 or MIT.
