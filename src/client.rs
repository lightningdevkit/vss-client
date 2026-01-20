use bitreq::Client;
use prost::Message;
use std::collections::HashMap;
use std::default::Default;
use std::sync::Arc;

use log::trace;

use crate::error::VssError;
use crate::headers::{FixedHeaders, VssHeaderProvider};
use crate::types::{
	DeleteObjectRequest, DeleteObjectResponse, GetObjectRequest, GetObjectResponse,
	ListKeyVersionsRequest, ListKeyVersionsResponse, PutObjectRequest, PutObjectResponse,
};
use crate::util::retry::{retry, RetryPolicy};
use crate::util::KeyValueVecKeyPrinter;

const APPLICATION_OCTET_STREAM: &str = "application/octet-stream";
const CONTENT_TYPE: &str = "content-type";
const DEFAULT_TIMEOUT_SECS: u64 = 10;
const MAX_RESPONSE_BODY_SIZE: usize = 1024 * 1024 * 1024; // 1GB
const DEFAULT_CLIENT_CAPACITY: usize = 10;

/// Thin-client to access a hosted instance of Versioned Storage Service (VSS).
/// The provided [`VssClient`] API is minimalistic and is congruent to the VSS server-side API.
#[derive(Clone)]
pub struct VssClient<R>
where
	R: RetryPolicy<E = VssError>,
{
	base_url: String,
	client: Client,
	retry_policy: R,
	header_provider: Arc<dyn VssHeaderProvider>,
}

impl<R: RetryPolicy<E = VssError>> VssClient<R> {
	/// Constructs a [`VssClient`] using `base_url` as the VSS server endpoint.
	pub fn new(base_url: String, retry_policy: R) -> Self {
		let client = Client::new(DEFAULT_CLIENT_CAPACITY);
		Self::from_client(base_url, client, retry_policy)
	}

	/// Constructs a [`VssClient`] from a given [`bitreq::Client`], using `base_url` as the VSS server endpoint.
	pub fn from_client(base_url: String, client: Client, retry_policy: R) -> Self {
		Self {
			base_url,
			client,
			retry_policy,
			header_provider: Arc::new(FixedHeaders::new(HashMap::new())),
		}
	}

	/// Constructs a [`VssClient`] from a given [`bitreq::Client`], using `base_url` as the VSS server endpoint.
	///
	/// HTTP headers will be provided by the given `header_provider`.
	pub fn from_client_and_headers(
		base_url: String, client: Client, retry_policy: R,
		header_provider: Arc<dyn VssHeaderProvider>,
	) -> Self {
		Self { base_url, client, retry_policy, header_provider }
	}

	/// Constructs a [`VssClient`] using `base_url` as the VSS server endpoint.
	///
	/// HTTP headers will be provided by the given `header_provider`.
	pub fn new_with_headers(
		base_url: String, retry_policy: R, header_provider: Arc<dyn VssHeaderProvider>,
	) -> Self {
		let client = Client::new(DEFAULT_CLIENT_CAPACITY);
		Self { base_url, client, retry_policy, header_provider }
	}

	/// Returns the underlying base URL.
	pub fn base_url(&self) -> &str {
		&self.base_url
	}

	/// Fetches a value against a given `key` in `request`.
	/// Makes a service call to the `GetObject` endpoint of the VSS server.
	/// For API contract/usage, refer to docs for [`GetObjectRequest`] and [`GetObjectResponse`].
	pub async fn get_object(
		&self, request: &GetObjectRequest,
	) -> Result<GetObjectResponse, VssError> {
		let request_id: u64 = rand::random();
		trace!("Sending GetObjectRequest {} for key {}.", request_id, request.key);
		let res = retry(
			|| async {
				let url = format!("{}/getObject", self.base_url);
				self.post_request(request, &url, true).await.and_then(
					|response: GetObjectResponse| {
						if response.value.is_none() {
							Err(VssError::InternalServerError(
								"VSS Server API Violation, expected value in GetObjectResponse but found none".to_string(),
							))
						} else {
							Ok(response)
						}
					},
				)
			},
			&self.retry_policy,
		)
		.await;
		if let Err(ref e) = res {
			trace!("GetObjectRequest {} failed: {}", request_id, e);
		}
		res
	}

	/// Writes multiple [`PutObjectRequest::transaction_items`] as part of a single transaction.
	/// Makes a service call to the `PutObject` endpoint of the VSS server, with multiple items.
	/// Items in the `request` are written in a single all-or-nothing transaction.
	/// For API contract/usage, refer to docs for [`PutObjectRequest`] and [`PutObjectResponse`].
	pub async fn put_object(
		&self, request: &PutObjectRequest,
	) -> Result<PutObjectResponse, VssError> {
		let request_id: u64 = rand::random();
		trace!(
			"Sending PutObjectRequest {} for transaction_items {} and delete_items {}.",
			request_id,
			KeyValueVecKeyPrinter(&request.transaction_items),
			KeyValueVecKeyPrinter(&request.delete_items),
		);
		let res = retry(
			|| async {
				let url = format!("{}/putObjects", self.base_url);
				self.post_request(request, &url, false).await
			},
			&self.retry_policy,
		)
		.await;
		if let Err(ref e) = res {
			trace!("PutObjectRequest {} failed: {}", request_id, e);
		}
		res
	}

	/// Deletes the given `key` and `value` in `request`.
	/// Makes a service call to the `DeleteObject` endpoint of the VSS server.
	/// For API contract/usage, refer to docs for [`DeleteObjectRequest`] and [`DeleteObjectResponse`].
	pub async fn delete_object(
		&self, request: &DeleteObjectRequest,
	) -> Result<DeleteObjectResponse, VssError> {
		let request_id: u64 = rand::random();
		trace!(
			"Sending DeleteObjectRequest {} for key {:?}",
			request_id,
			request.key_value.as_ref().map(|t| &t.key)
		);
		let res = retry(
			|| async {
				let url = format!("{}/deleteObject", self.base_url);
				self.post_request(request, &url, true).await
			},
			&self.retry_policy,
		)
		.await;
		if let Err(ref e) = res {
			trace!("DeleteObjectRequest {} failed: {}", request_id, e);
		}
		res
	}

	/// Lists keys and their corresponding version for a given [`ListKeyVersionsRequest::store_id`].
	/// Makes a service call to the `ListKeyVersions` endpoint of the VSS server.
	/// For API contract/usage, refer to docs for [`ListKeyVersionsRequest`] and [`ListKeyVersionsResponse`].
	pub async fn list_key_versions(
		&self, request: &ListKeyVersionsRequest,
	) -> Result<ListKeyVersionsResponse, VssError> {
		let request_id: u64 = rand::random();
		trace!(
			"Sending ListKeyVersionsRequest {} for key_prefix {:?}, page_size {:?}, page_token {:?}",
			request_id,
			request.key_prefix,
			request.page_size,
			request.page_token
		);
		let res = retry(
			|| async {
				let url = format!("{}/listKeyVersions", self.base_url);
				self.post_request(request, &url, true).await
			},
			&self.retry_policy,
		)
		.await;
		if let Err(ref e) = res {
			trace!("ListKeyVersionsRequest {} failed: {}", request_id, e);
		}
		res
	}

	async fn post_request<Rq: Message, Rs: Message + Default>(
		&self, request: &Rq, url: &str, enable_pipelining: bool,
	) -> Result<Rs, VssError> {
		let request_body = request.encode_to_vec();
		let headers = self
			.header_provider
			.get_headers(&request_body)
			.await
			.map_err(|e| VssError::AuthError(e.to_string()))?;

		let mut http_request = bitreq::post(url)
			.with_header(CONTENT_TYPE, APPLICATION_OCTET_STREAM)
			.with_headers(headers)
			.with_body(request_body)
			.with_timeout(DEFAULT_TIMEOUT_SECS)
			.with_max_body_size(Some(MAX_RESPONSE_BODY_SIZE));

		if enable_pipelining {
			http_request = http_request.with_pipelining();
		}

		let response = self.client.send_async(http_request).await?;

		let status_code = response.status_code;
		let payload = response.into_bytes();

		if (200..300).contains(&status_code) {
			let response = Rs::decode(&payload[..])?;
			Ok(response)
		} else {
			Err(VssError::new(status_code, payload))
		}
	}
}
