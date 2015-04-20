use std::string;
use std::fmt;
use std::io;

use byteorder;

use service;
use util::ReadCStringWithLenError;

/// Errors returned by `IdentityService::get_default_ego`. 
#[derive(Debug)]
pub enum GetDefaultEgoError {
  /// The name of the service was too long.
  NameTooLong(String),
  /// An I/O error occured while communicating with the identity service.
  Io(io::Error),
  /// Failed to read a message from the server.
  ReadMessage(service::ReadMessageError),
  /// The service responded with an error message.
  ServiceResponse(String),
  /// The service responded with an error message but the message contained invalid utf-8.
  MalformedErrorResponse(string::FromUtf8Error),
  /// Failed to receive the identity name from the service.
  ReceiveName(ReadCStringWithLenError),
  /// Failed to connect to the identity service.
  Connect(ConnectError),
  /// The service response was incoherent. You should file a bug-report if you encounter this
  /// variant.
  InvalidResponse,
  /// The remote service disconnected.
  Disconnected,
}
error_chain! {ConnectError, GetDefaultEgoError, Connect}
error_chain! {io::Error, GetDefaultEgoError, Io}
error_chain! {service::ReadMessageError, GetDefaultEgoError, ReadMessage}
error_chain! {ReadCStringWithLenError, GetDefaultEgoError, ReceiveName}
byteorder_error_chain! {GetDefaultEgoError}

/// Errors returned by `IdentityService::connect`
#[derive(Debug)]
pub enum ConnectError {
  /// Failed to connect to the service.
  Connect(service::ConnectError),
  /// The remote service disconnected.
  Disconnected,
  /// There was an I/O error communicating with the service.
  Io(io::Error),
  /// Failed to read a message from the service.
  ReadMessage(service::ReadMessageError),
  /// The service responded with an invalid utf-8 name. *(It is a bug to see this variant)*
  InvalidName(string::FromUtf8Error),
  /// Received an unexpected message from the service. *(It is a bug to see this variant)*
  UnexpectedMessageType(u16),
}
error_chain! {service::ConnectError, ConnectError, Connect}
error_chain! {io::Error, ConnectError, Io}
error_chain! {service::ReadMessageError, ConnectError, ReadMessage}
byteorder_error_chain! {ConnectError}

/// Errors returned by `identity::get_default_ego`
#[derive(Debug)]
pub enum ConnectGetDefaultEgoError {
  /// Ego lookup failed.
  GetDefaultEgo(GetDefaultEgoError),
  /// Failed to connect to the service and perform initialization.
  Connect(ConnectError),
}
error_chain! {GetDefaultEgoError, ConnectGetDefaultEgoError, GetDefaultEgo}
error_chain! {ConnectError, ConnectGetDefaultEgoError, Connect}

impl fmt::Display for GetDefaultEgoError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &GetDefaultEgoError::NameTooLong(ref s)
          => write!(f, "Name of service \"{}\" is too long for default ego lookup", s),
      &GetDefaultEgoError::Io(ref e)
          => write!(f, "I/O error communicating with identity service during default ego lookup: {}", e),
      &GetDefaultEgoError::ReadMessage(ref e)
          => write!(f, "Error receiving message from identity service during default ego lookup: {}", e),
      &GetDefaultEgoError::ServiceResponse(ref s)
          => write!(f, "Service responded with an error message in response to default ego lookup: {}", s),
      &GetDefaultEgoError::MalformedErrorResponse(ref e)
          => write!(f, "Service responded with an error message in response to default ego lookup but the response contained invalid utf-8: {}", e),
      &GetDefaultEgoError::ReceiveName(ref e)
          => write!(f, "Failed to receive the identity name from the service during default ego lookup: {}", e),
      &GetDefaultEgoError::Connect(ref e)
          => write!(f, "Failed to connect to identity service for default ego lookup: {}", e),
      &GetDefaultEgoError::InvalidResponse
          => write!(f, "Service response was incoherent. THIS IS A BUG! Please file a bug report at {}", ::HOMEPAGE),
      &GetDefaultEgoError::Disconnected
          => write!(f, "The service unexpectedly disconnected."),
    }
  }
}

impl fmt::Display for ConnectError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &ConnectError::Connect(ref e)
          => write!(f, "Failed to contact the identity service: {}", e),
      &ConnectError::Io(ref e)
          => write!(f, "I/O error communicating with the identity service during initial exchange: {}", e),
      &ConnectError::ReadMessage(ref e)
          => write!(f, "Error receiving message from identity service during connection: {}", e),
      &ConnectError::InvalidName(ref e)
          => write!(f, "The identity service sent a non-utf8 encoded name during initial exchange when connecting ({}). THIS IS A BUG. Please file a bug report at {}", e, ::HOMEPAGE),
      &ConnectError::UnexpectedMessageType(n)
          => write!(f, "The identity service sent an unexpected message type ({}) during initial exchange when connecting. THIS IS A BUG. Please file a bug report at {}", n, ::HOMEPAGE),
      &ConnectError::Disconnected
          => write!(f, "The service unexpectedly disconnected."),
    }
  }
}

impl fmt::Display for ConnectGetDefaultEgoError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &ConnectGetDefaultEgoError::GetDefaultEgo(ref e)
          => write!(f, "Connected to identity service but default ego lookup failed: {}", e),
      &ConnectGetDefaultEgoError::Connect(ref e)
          => write!(f, "Failed to connect to identity service to perform default ego lookup: {}", e),
    }
  }
}

