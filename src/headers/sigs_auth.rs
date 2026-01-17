//! Provides the [`SigsAuthProvider`].

use crate::headers::{VssHeaderProvider, VssHeaderProviderError};
use async_trait::async_trait;
use bitcoin::hashes::sha256::Hash as Sha256;
use bitcoin::hashes::Hash as _;
use bitcoin::secp256k1::{Message, Secp256k1, SecretKey, SignOnly};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::Write as _;
use std::time::SystemTime;

/// A 64-byte constant which, after appending the public key, is signed in order to prove knowledge
/// of the corresponding private key.
pub const SIGNING_CONSTANT: &'static [u8] =
	b"VSS Signature Authorizer Signing Salt Constant..................";

fn build_token(secret_key: &SecretKey, secp_ctx: &Secp256k1<SignOnly>) -> String {
	let pubkey = secret_key.public_key(secp_ctx);
	let old_time = "System time must be at least Jan 1, 1970";
	let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect(old_time).as_secs();

	// 2^64 serialized as a string is 20 bytes.
	let mut buffer = [0u8; SIGNING_CONSTANT.len() + 33 + 20];
	let mut stream = &mut buffer[..];
	stream.write_all(SIGNING_CONSTANT).unwrap();
	stream.write_all(&pubkey.serialize()).unwrap();
	write!(stream, "{now}").unwrap();
	let bytes_remaining = stream.len();
	let bytes_to_sign = &buffer[..buffer.len() - bytes_remaining];

	let hash = Sha256::hash(&bytes_to_sign);
	let sig = secp_ctx.sign_ecdsa(&Message::from_digest(hash.to_byte_array()), secret_key);
	let mut out = String::with_capacity((33 + 64 + 20) * 2);
	write!(&mut out, "{pubkey:x}").unwrap();
	for c in sig.serialize_compact() {
		write!(&mut out, "{:02x}", c).unwrap();
	}
	write!(&mut out, "{now}").unwrap();
	out
}

/// A simple auth provider which simply proves knowledge of a private key.
///
/// It provides a good default authentication mechanism for testing, or in the case that
/// denial-of-service protection against new-account-flooding is mitigated at another layer
/// (e.g. via Apple DeviceCheck or similar remote attestation technologies).
pub struct SigsAuthProvider {
	key: SecretKey,
	secp_ctx: Secp256k1<SignOnly>,
	default_headers: HashMap<String, String>,
}

impl SigsAuthProvider {
	/// Creates a new auth provider which simply proves knowledge of a private key.
	///
	/// This provides an incredibly simple authentication scheme and allows the server to ensure
	/// data for separate clients is kept separate, without any application-specific logic.
	///
	/// In addition to the automatically-added `Authorization` header, any headers provided in
	/// `default_headers` (except an `Authorization` header) will be added to the headers list.
	pub fn new(key: SecretKey, default_headers: HashMap<String, String>) -> Self {
		SigsAuthProvider { secp_ctx: Secp256k1::signing_only(), key, default_headers }
	}
}

#[async_trait]
impl VssHeaderProvider for SigsAuthProvider {
	async fn get_headers(
		&self, _request: &[u8],
	) -> Result<HashMap<String, String>, VssHeaderProviderError> {
		// TODO: We might consider not re-signing on every request, but its cheap enough that it
		// doesn't really matter
		let mut headers = self.default_headers.clone();
		headers.insert("Authorization".to_owned(), build_token(&self.key, &self.secp_ctx));
		Ok(headers)
	}
}
