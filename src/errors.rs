pub(crate) fn to_napi_error(error: String) -> napi::Error {
  napi::Error::new(napi::Status::GenericFailure, error)
}
